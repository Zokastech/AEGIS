// AEGIS — zokastech.fr — Apache 2.0 / MIT

package rest

import (
	"context"
	"encoding/json"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/labstack/echo/v4"
	"github.com/zokastech/aegis/aegis-gateway/policy"
)

// analyzePolicyResponse wraps engine output plus policy metadata (when a policy is active).
type analyzePolicyResponse struct {
	Result       json.RawMessage            `json:"result"`
	Minimization *policy.MinimizationReport `json:"minimization,omitempty"`
	PolicyReport *policy.PolicyReport       `json:"policy_report,omitempty"`
}

func policyNameFromRequest(c echo.Context, bodyName string) string {
	if strings.TrimSpace(bodyName) != "" {
		return strings.TrimSpace(bodyName)
	}
	return strings.TrimSpace(c.QueryParam("policy"))
}

// ListPolicies GET /v1/policies
func ListPolicies(s *Services) echo.HandlerFunc {
	return func(c echo.Context) error {
		if s.Policy == nil {
			return errJSON(c, http.StatusServiceUnavailable, "POLICY", "moteur de politiques non chargé")
		}
		names := s.Policy.ListNames()
		return c.JSON(http.StatusOK, map[string]interface{}{"policies": names})
	}
}

// GetDPIA GET /v1/policy/dpia?policy=&entity_types=&format=markdown|html
func GetDPIA(s *Services) echo.HandlerFunc {
	return func(c echo.Context) error {
		if s.Policy == nil {
			return errJSON(c, http.StatusServiceUnavailable, "POLICY", "moteur de politiques non chargé")
		}
		pname := strings.TrimSpace(c.QueryParam("policy"))
		if pname == "" {
			return errJSON(c, http.StatusBadRequest, "VALIDATION", "query policy requis")
		}
		pol, err := s.Policy.Policy(pname)
		if err != nil {
			return errJSON(c, http.StatusNotFound, "POLICY", err.Error())
		}
		if !pol.Rights.DpiaAutoReport {
			return errJSON(c, http.StatusForbidden, "POLICY", "DPIA auto désactivé pour cette politique")
		}
		var types []string
		if raw := c.QueryParam("entity_types"); raw != "" {
			for _, t := range strings.Split(raw, ",") {
				t = strings.TrimSpace(t)
				if t != "" {
					types = append(types, t)
				}
			}
		}
		if len(types) == 0 {
			for _, e := range pol.Entities {
				types = append(types, e.Type)
			}
		}
		rep := policy.BuildDPIA(pol, types)
		format := policy.DPIAFormatMarkdown
		switch strings.ToLower(strings.TrimSpace(c.QueryParam("format"))) {
		case "html":
			format = policy.DPIAFormatHTML
		}
		body, ct := policy.WriteDPIA(rep, format)
		return c.Blob(http.StatusOK, ct, []byte(body))
	}
}

// DeleteSubject DELETE /v1/subjects/:id?policy=
func DeleteSubject(s *Services) echo.HandlerFunc {
	return func(c echo.Context) error {
		if s.Policy == nil {
			return errJSON(c, http.StatusServiceUnavailable, "POLICY", "moteur de politiques non chargé")
		}
		subjectID := strings.TrimSpace(c.Param("id"))
		pname := strings.TrimSpace(c.QueryParam("policy"))
		if pname == "" {
			return errJSON(c, http.StatusBadRequest, "VALIDATION", "query policy requis")
		}
		svc := policy.NewErasureService(s.Policy)
		ctx, cancel := context.WithTimeout(c.Request().Context(), 15*time.Second)
		defer cancel()
		cert, err := svc.EraseSubject(ctx, pname, subjectID)
		if err != nil {
			if err == policy.ErrErasureDisabled {
				return errJSON(c, http.StatusForbidden, "ERASURE", "effacement désactivé pour cette politique")
			}
			if err == policy.ErrPolicyNotFound {
				return errJSON(c, http.StatusNotFound, "POLICY", err.Error())
			}
			return errJSON(c, http.StatusInternalServerError, "ERASURE", err.Error())
		}
		b, _ := cert.ToJSON()
		return c.Blob(http.StatusOK, "application/json", b)
	}
}

// registerPseudonymMapping when subject and policy include retention.
func registerPseudonymMapping(s *Services, ctx context.Context, polName, subjectID string) {
	if s.Policy == nil || subjectID == "" {
		return
	}
	pol, err := s.Policy.Policy(polName)
	if err != nil || pol.Retention.PseudonymizationMappingDays <= 0 {
		return
	}
	key := uuid.NewString()
	exp := time.Now().Add(time.Duration(pol.Retention.PseudonymizationMappingDays) * 24 * time.Hour)
	_ = s.Policy.MappingStore().Register(ctx, subjectID, key, &exp)
}

// mergePolicyAnonymizeConfig merges policy into engine JSON config.
func mergePolicyAnonymizeConfig(s *Services, polName, userJSON string) (string, error) {
	if s.Policy == nil || polName == "" {
		return userJSON, nil
	}
	pol, err := s.Policy.Policy(polName)
	if err != nil {
		return userJSON, err
	}
	return policy.MergeAnonymizeConfig(userJSON, pol)
}

// applyPolicyAfterAnalyze mutates the analyze result JSON.
func applyPolicyAfterAnalyze(s *Services, ctx context.Context, polName string, raw []byte) ([]byte, *policy.PolicyReport, error) {
	if s.Policy == nil || polName == "" {
		return raw, nil, nil
	}
	pol, err := s.Policy.Policy(polName)
	if err != nil {
		return nil, nil, err
	}
	return s.Policy.AfterAnalyze(ctx, pol, raw)
}

// applyPolicyBeforeAnalyze returns minimized text.
func applyPolicyBeforeAnalyze(s *Services, ctx context.Context, polName, text string) (string, *policy.MinimizationReport, error) {
	if s.Policy == nil || polName == "" {
		return text, nil, nil
	}
	pol, err := s.Policy.Policy(polName)
	if err != nil {
		return "", nil, err
	}
	return s.Policy.BeforeAnalyze(ctx, pol, text)
}

