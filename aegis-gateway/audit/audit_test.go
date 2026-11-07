// AEGIS — zokastech.fr — Apache 2.0 / MIT

package audit

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestFileLoggerChainAndPermissions(t *testing.T) {
	dir := t.TempDir()
	p := filepath.Join(dir, "a", "audit.jsonl")
	l := NewFileLogger(p, 0, "")
	for i := 0; i < 5; i++ {
		if err := l.Append(Entry{Actor: "api_key:k", Action: "POST /v1/analyze", Success: true, StatusCode: 200}); err != nil {
			t.Fatal(err)
		}
	}
	st, err := os.Stat(p)
	if err != nil {
		t.Fatal(err)
	}
	if st.Mode().Perm() != 0o600 {
		t.Fatalf("permissions journal %o", st.Mode().Perm())
	}
	lines, err := ScanLinesForward(p, 100)
	if err != nil || len(lines) != 5 {
		t.Fatalf("%d lignes, %v", len(lines), err)
	}
	if idx, err := VerifyChain(lines, ""); idx != -1 || err != nil {
		t.Fatalf("chaîne valide rejetée: %d %v", idx, err)
	}
	// Tamper with line 2.
	lines[2] = strings.Replace(lines[2], "api_key:k", "api_key:attacker", 1)
	if idx, _ := VerifyChain(lines, ""); idx != 2 {
		t.Fatalf("altération non détectée (idx=%d)", idx)
	}
	// Remove a line.
	full, _ := ScanLinesForward(p, 100)
	if idx, _ := VerifyChain(append(full[:1], full[2:]...), ""); idx != 1 {
		t.Fatalf("suppression non détectée (idx=%d)", idx)
	}
}

func TestRecoverLastHashAcrossRestart(t *testing.T) {
	p := filepath.Join(t.TempDir(), "audit.jsonl")
	l := NewFileLogger(p, 0, "")
	_ = l.Append(Entry{Actor: "a"})
	l2 := NewFileLogger(p, 0, "")
	if l2.LastHash() != l.LastHash() || l2.LastHash() == "" {
		t.Fatal("dernier hash non récupéré au redémarrage")
	}
	_ = l2.Append(Entry{Actor: "b"})
	lines, _ := ScanLinesForward(p, 10)
	if idx, err := VerifyChain(lines, ""); idx != -1 {
		t.Fatalf("chaîne rompue après redémarrage: %v", err)
	}
}

func TestSIEMExportEscaping(t *testing.T) {
	e := Entry{Actor: "x\tsev=0\nLEEF:2.0|fake", Endpoint: "/v1/a|b=c\r\nCEF:0|evil", Action: "GET /x", Success: false}
	leef := ToLEEF(e)
	if strings.Count(leef, "\n") != 0 || strings.Count(leef, "\r") != 0 || strings.Contains(leef, "\tsev=0") {
		t.Fatalf("injection LEEF: %q", leef)
	}
	cef := ToCEF(e)
	if strings.ContainsAny(cef, "\r\n") {
		t.Fatalf("injection CEF: %q", cef)
	}
	if !strings.HasPrefix(cef, "CEF:0|zokastech|") || !strings.Contains(cef, "|8|") {
		t.Fatalf("CEF mal formé: %q", cef)
	}
}
