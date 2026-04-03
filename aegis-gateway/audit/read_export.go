// AEGIS — zokastech.fr — Apache 2.0 / MIT

package audit

import (
	"bufio"
	"os"
)

// ScanLinesForward reads the file and keeps the last maxLines non-empty lines (moderate-sized audit files).
func ScanLinesForward(path string, maxLines int) ([]string, error) {
	if maxLines <= 0 {
		maxLines = 1000
	}
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	sc := bufio.NewScanner(f)
	const maxScan = 10 * 1024 * 1024
	buf := make([]byte, 0, 64*1024)
	sc.Buffer(buf, maxScan)
	var ring []string
	for sc.Scan() {
		line := sc.Text()
		if line == "" {
			continue
		}
		if len(ring) >= maxLines {
			ring = ring[1:]
		}
		ring = append(ring, line)
	}
	return ring, sc.Err()
}
