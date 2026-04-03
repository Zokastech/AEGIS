// AEGIS — zokastech.fr — Apache 2.0 / MIT

package policy

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"time"
)

// ErasureService orchestrates Art. 17 / LGPD erasure.
type ErasureService struct {
	Engine *PolicyEngine
}

// NewErasureService wraps the policy engine.
func NewErasureService(e *PolicyEngine) *ErasureService {
	return &ErasureService{Engine: e}
}

// EraseSubject removes pseudonym mappings for a subject; policyName must allow erasure.
func (s *ErasureService) EraseSubject(ctx context.Context, policyName, subjectID string) (*ErasureCertificate, error) {
	if subjectID == "" {
		return nil, fmt.Errorf("erasure: subject_id vide")
	}
	pol, err := s.Engine.Policy(policyName)
	if err != nil {
		return nil, err
	}
	if !pol.Rights.ErasureEndpointEnabled {
		return nil, ErrErasureDisabled
	}
	n, err := s.Engine.MappingStore().DeleteBySubject(ctx, subjectID)
	if err != nil {
		return nil, err
	}
	cert := NewErasureCertificate(subjectID, policyName, string(pol.Regulation), n)
	return cert, nil
}

// ErasureCertificate is proof of erasure (legal record of the act, not the erased data).
type ErasureCertificate struct {
	SubjectID       string    `json:"subject_id"`
	Policy          string    `json:"policy"`
	Regulation      string    `json:"regulation"`
	MappingsRemoved int       `json:"mappings_removed"`
	IssuedAt        time.Time `json:"issued_at"`
	CertificateID   string    `json:"certificate_id"`
	IntegrityHash   string    `json:"integrity_hash_sha256"`
}

// NewErasureCertificate builds the certificate and an integrity hash over the canonical payload.
func NewErasureCertificate(subjectID, policy, regulation string, removed int) *ErasureCertificate {
	issued := time.Now().UTC()
	base := fmt.Sprintf("%s|%s|%s|%d|%s", subjectID, policy, regulation, removed, issued.Format(time.RFC3339Nano))
	h := sha256.Sum256([]byte(base))
	id := hex.EncodeToString(h[:16])
	full := sha256.Sum256([]byte("AEGIS_ERASURE_V1|" + base + "|" + id))
	return &ErasureCertificate{
		SubjectID:       subjectID,
		Policy:          policy,
		Regulation:      regulation,
		MappingsRemoved: removed,
		IssuedAt:        issued,
		CertificateID:   id,
		IntegrityHash:   hex.EncodeToString(full[:]),
	}
}

// ToJSON serializes the certificate.
func (c *ErasureCertificate) ToJSON() ([]byte, error) {
	return json.MarshalIndent(c, "", "  ")
}
