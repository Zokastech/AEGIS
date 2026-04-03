// AEGIS — zokastech.fr — Apache 2.0 / MIT

package audit

import (
	"encoding/json"
	"os"
	"path/filepath"
	"sync"
	"time"
)

// FileLogger is an append-only JSONL audit log with hash chaining.
type FileLogger struct {
	mu       sync.Mutex
	path     string
	lastHash string
	maxBytes int64
	archive  string
}

// NewFileLogger starts with empty lastHash (genesis).
func NewFileLogger(path string, maxMB int, archiveDir string) *FileLogger {
	if maxMB <= 0 {
		maxMB = 100
	}
	_ = os.MkdirAll(filepath.Dir(path), 0o750)
	if archiveDir != "" {
		_ = os.MkdirAll(archiveDir, 0o750)
	}
	fl := &FileLogger{
		path:     path,
		lastHash: "",
		maxBytes: int64(maxMB) * 1024 * 1024,
		archive:  archiveDir,
	}
	fl.recoverLastHash()
	return fl
}

func (f *FileLogger) recoverLastHash() {
	b, err := os.ReadFile(f.path)
	if err != nil || len(b) == 0 {
		return
	}
	lines := splitLinesKeepLastNonEmpty(b)
	if len(lines) == 0 {
		return
	}
	var last Entry
	if json.Unmarshal(lines[len(lines)-1], &last) == nil && last.EntryHashHex != "" {
		f.lastHash = last.EntryHashHex
	}
}

func splitLinesKeepLastNonEmpty(b []byte) [][]byte {
	var out [][]byte
	start := 0
	for i := range b {
		if b[i] == '\n' {
			line := b[start:i]
			if len(line) > 0 {
				out = append(out, line)
			}
			start = i + 1
		}
	}
	if start < len(b) && len(b[start:]) > 0 {
		out = append(out, b[start:])
	}
	return out
}

// Append writes one JSON line and updates the hash chain.
func (f *FileLogger) Append(e Entry) error {
	f.mu.Lock()
	defer f.mu.Unlock()
	e.PrevHashHex = f.lastHash
	eh, err := ChainHash(f.lastHash, e)
	if err != nil {
		return err
	}
	e.EntryHashHex = eh
	line, err := json.Marshal(e)
	if err != nil {
		return err
	}
	fi, statErr := os.Stat(f.path)
	if statErr == nil && f.maxBytes > 0 && fi.Size()+int64(len(line))+1 >= f.maxBytes {
		if err := f.rotateLocked(); err != nil {
			return err
		}
	}
	fh, err := os.OpenFile(f.path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o600)
	if err != nil {
		return err
	}
	defer fh.Close()
	if _, err := fh.Write(append(line, '\n')); err != nil {
		return err
	}
	f.lastHash = eh
	return nil
}

func (f *FileLogger) rotateLocked() error {
	if f.archive == "" {
		return nil
	}
	arch := filepath.Join(f.archive, "audit_"+time.Now().UTC().Format("20060102T150405")+".jsonl")
	if err := os.Rename(f.path, arch); err != nil && !os.IsNotExist(err) {
		return err
	}
	// Keep lastHash: the next entry chains from the last line of the archived file.
	return nil
}

// LastHash is exposed for tests / external verification.
func (f *FileLogger) LastHash() string {
	f.mu.Lock()
	defer f.mu.Unlock()
	return f.lastHash
}
