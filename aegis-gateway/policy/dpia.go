// AEGIS — zokastech.fr — Apache 2.0 / MIT

package policy

import (
	"fmt"
	"sort"
	"strings"
	"time"
)

// DPIAExportFormat selects report output format.
type DPIAExportFormat string

const (
	DPIAFormatMarkdown DPIAExportFormat = "markdown"
	DPIAFormatHTML     DPIAExportFormat = "html"
)

// DPIAReport is an automated impact-assessment summary.
type DPIAReport struct {
	Title          string              `json:"title"`
	GeneratedAt    time.Time           `json:"generated_at"`
	PolicyName     string              `json:"policy_name"`
	Regulation     string              `json:"regulation"`
	ProcessingDesc string              `json:"processing_description"`
	DataCategories []string            `json:"data_categories"`
	RiskScore      int                 `json:"risk_score"` // 0–100
	RiskBand       string              `json:"risk_band"`
	Measures       []string            `json:"measures"`
	ResidualRisks  []string            `json:"residual_risks"`
	ByCategory     map[string]int      `json:"risk_by_category"`
}

// BuildDPIA builds a DPIA from a policy and the entity types actually processed.
func BuildDPIA(pol *PolicyDocument, detectedEntityTypes []string) *DPIAReport {
	r := &DPIAReport{
		GeneratedAt:    time.Now().UTC(),
		PolicyName:     pol.Name,
		Regulation:     string(pol.Regulation),
		ProcessingDesc: pol.Description,
		DataCategories: dedupeSorted(detectedEntityTypes),
		Measures:       defaultMeasures(pol),
		ResidualRisks:  residualRisks(pol),
		ByCategory:     map[string]int{},
	}
	r.Title = fmt.Sprintf("DPIA — %s (%s)", pol.Name, pol.Regulation)
	r.RiskScore, r.RiskBand, r.ByCategory = computeRiskScore(pol, detectedEntityTypes)
	return r
}

func dedupeSorted(in []string) []string {
	seen := map[string]struct{}{}
	var out []string
	for _, s := range in {
		s = strings.TrimSpace(s)
		if s == "" {
			continue
		}
		if _, ok := seen[s]; ok {
			continue
		}
		seen[s] = struct{}{}
		out = append(out, s)
	}
	sort.Strings(out)
	return out
}

// entitySensitivity is per-type weights for risk scoring.
var entitySensitivity = map[string]int{
	"MEDICAL_RECORD": 35,
	"SSN":            30,
	"PASSPORT":       28,
	"NATIONAL_ID":    28,
	"CREDIT_CARD":    25,
	"IBAN":           22,
	"BANK_ACCOUNT":   22,
	"TAX_ID":         20,
	"EMAIL":          8,
	"PHONE":          8,
	"PERSON":         12,
	"ADDRESS":        14,
	"LOCATION":       10,
	"ORGANIZATION":   6,
	"IP_ADDRESS":     7,
	"CRYPTO_WALLET":  15,
	"DRIVER_LICENSE": 25,
}

func computeRiskScore(pol *PolicyDocument, types []string) (int, string, map[string]int) {
	by := make(map[string]int)
	base := 5
	for _, t := range types {
		w, ok := entitySensitivity[t]
		if !ok {
			w = 10
		}
		by[t] = w
		base += w
	}
	if pol.Defaults.BlockHighRiskCombinations {
		base += 5
	}
	if pol.Retention.PseudonymizationMappingDays > 90 {
		base += 10
	}
	if pol.DataMinimization.Enabled {
		base -= 8
	}
	if base < 0 {
		base = 0
	}
	if base > 100 {
		base = 100
	}
	band := "faible"
	if base >= 40 {
		band = "modéré"
	}
	if base >= 65 {
		band = "élevé"
	}
	if base >= 85 {
		band = "critique"
	}
	return base, band, by
}

func defaultMeasures(pol *PolicyDocument) []string {
	out := []string{
		"Chiffrement en transit (TLS) sur le gateway",
		"Contrôle d’accès RBAC et journal d’audit",
	}
	if pol.DataMinimization.Enabled {
		out = append(out, "Minimisation des données à l’ingestion (art. 25 RGPD)")
	}
	if pol.Retention.PseudonymizationMappingDays == 0 {
		out = append(out, "Aucune rétention prolongée des mappings de pseudonymisation")
	} else {
		out = append(out, fmt.Sprintf("TTL mappings pseudonymisation : %d j", pol.Retention.PseudonymizationMappingDays))
	}
	return out
}

func residualRisks(pol *PolicyDocument) []string {
	r := []string{"Erreur de configuration opérateur / politique", "Fuite latérale via métadonnées de logs"}
	if pol.Regulation == RegHIPAA {
		r = append(r, "PHI hors périmètre technique (supports papier, dictée)")
	}
	return r
}

// WriteDPIA renders the report (Markdown or print-friendly HTML → PDF in the browser).
func WriteDPIA(r *DPIAReport, f DPIAExportFormat) (string, string) {
	switch f {
	case DPIAFormatHTML:
		return r.toHTML(), "text/html; charset=utf-8"
	default:
		return r.toMarkdown(), "text/markdown; charset=utf-8"
	}
}

func (r *DPIAReport) toMarkdown() string {
	var b strings.Builder
	b.WriteString("# " + r.Title + "\n\n")
	b.WriteString(fmt.Sprintf("**Date** : %s  \n", r.GeneratedAt.Format(time.RFC3339)))
	b.WriteString(fmt.Sprintf("**Politique** : `%s`  \n", r.PolicyName))
	b.WriteString(fmt.Sprintf("**Cadre** : %s  \n\n", r.Regulation))
	b.WriteString("## Description du traitement\n\n")
	b.WriteString(r.ProcessingDesc + "\n\n")
	b.WriteString("## Catégories de données\n\n")
	for _, c := range r.DataCategories {
		score := r.ByCategory[c]
		b.WriteString(fmt.Sprintf("- `%s` (score composant : %d)\n", c, score))
	}
	b.WriteString("\n## Score de risque global\n\n")
	b.WriteString(fmt.Sprintf("**%d / 100** — bande **%s**\n\n", r.RiskScore, r.RiskBand))
	b.WriteString("## Mesures\n\n")
	for _, m := range r.Measures {
		b.WriteString("- " + m + "\n")
	}
	b.WriteString("\n## Risques résiduels\n\n")
	for _, x := range r.ResidualRisks {
		b.WriteString("- " + x + "\n")
	}
	b.WriteString("\n---\n*Document généré automatiquement par AEGIS (zokastech.fr) — revue humaine requise.*\n")
	return b.String()
}

func (r *DPIAReport) toHTML() string {
	md := r.toMarkdown()
	// Minimal markdown→HTML for headings and lists (no external dependency).
	lines := strings.Split(md, "\n")
	var out strings.Builder
	out.WriteString("<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>")
	out.WriteString(htmlEscape(r.Title))
	out.WriteString("</title><style>body{font-family:system-ui,sans-serif;max-width:800px;margin:2rem auto;}code{background:#f4f4f4;padding:2px 4px;}</style></head><body>")
	for _, line := range lines {
		line = strings.TrimRight(line, " ")
		switch {
		case strings.HasPrefix(line, "# "):
			out.WriteString("<h1>" + htmlEscape(strings.TrimPrefix(line, "# ")) + "</h1>")
		case strings.HasPrefix(line, "## "):
			out.WriteString("<h2>" + htmlEscape(strings.TrimPrefix(line, "## ")) + "</h2>")
		case strings.HasPrefix(line, "- "):
			out.WriteString("<li>" + htmlEscape(strings.TrimPrefix(line, "- ")) + "</li>")
		case strings.HasPrefix(line, "---"):
			out.WriteString("<hr/>")
		case line == "":
			out.WriteString("<br/>")
		default:
			s := line
			s = strings.ReplaceAll(s, "**", "")
			out.WriteString("<p>" + htmlEscape(s) + "</p>")
		}
	}
	out.WriteString("</body></html>")
	return out.String()
}

func htmlEscape(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	s = strings.ReplaceAll(s, "\"", "&quot;")
	return s
}
