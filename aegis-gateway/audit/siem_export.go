// AEGIS — zokastech.fr — Apache 2.0 / MIT

package audit

import (
	"fmt"
	"strings"
)

// ToCEF converts an entry to ArcSight CEF format (subset).
func ToCEF(e Entry) string {
	// CEF:Version|Device Vendor|Device Product|Device Version|Signature ID|Name|Severity|extension
	sev := "3"
	if !e.Success {
		sev = "8"
	}
	name := strings.ReplaceAll(e.Action, "|", "/")
	ext := fmt.Sprintf("requestId=%s endpt=%s method=%s code=%d actor=%s auth=%s prevHash=%s entryHash=%s",
		escapeCEF(e.RequestID), escapeCEF(e.Endpoint), escapeCEF(e.Method), e.StatusCode,
		escapeCEF(e.Actor), escapeCEF(e.AuthMethod), e.PrevHashHex, e.EntryHashHex)
	return fmt.Sprintf("CEF:0|zokastech|AEGIS-Gateway|1.0|%s|%s|%s|%s",
		escapeCEF(e.Action), escapeCEF(name), sev, ext)
}

func escapeCEF(s string) string {
	s = strings.ReplaceAll(s, `\`, `\\`)
	s = strings.ReplaceAll(s, `=`, `\=`)
	s = strings.ReplaceAll(s, "|", `\|`)
	s = strings.ReplaceAll(s, "\n", " ")
	return s
}

// ToLEEF converts an entry to IBM LEEF 2.0 format (subset).
func ToLEEF(e Entry) string {
	sev := "3"
	if !e.Success {
		sev = "8"
	}
	return fmt.Sprintf("LEEF:2.0|zokastech|AEGIS|1.0|%s|devTime=%s\tsev=%s\tusr=%s\tauth=%s\turi=%s\tmethod=%s\tsuccess=%t\tcode=%d\trid=%s",
		e.Action, e.TimestampRFC3339, sev, e.Actor, e.AuthMethod, e.Endpoint, e.Method, e.Success, e.StatusCode, e.RequestID)
}
