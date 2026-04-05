// AEGIS — zokastech.fr — Apache 2.0 / MIT

package app

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"time"

	"github.com/zokastech/aegis/aegis-gateway/internal/domain"
	"github.com/zokastech/aegis/aegis-gateway/policy"
)

// Phase errors map to HTTP status codes (before = 400 POLICY unless blocked; after = 500 POLICY unless blocked).
var (
	ErrPolicyPhaseBefore = errors.New("analyze: politique avant moteur")
	ErrPolicyPhaseAfter  = errors.New("analyze: politique après moteur")
)

// Analyze is the PII analysis use case with optional policy and entity-type filter (RBAC).
type Analyze struct {
	Pool    EnginePool
	Circuit Circuit
	Policy  PolicyGate
	Timeout time.Duration
}

// NewAnalyze constructs the use case; policy may be nil via PolicyEngineGate{Eng: nil}.
func NewAnalyze(pool EnginePool, circuit Circuit, pol PolicyGate, engineTimeout time.Duration) *Analyze {
	if pol == nil {
		pol = PolicyEngineGate{}
	}
	return &Analyze{
		Pool:    pool,
		Circuit: circuit,
		Policy:  pol,
		Timeout: engineTimeout,
	}
}

// AnalyzeCommand is the use case input (independent of Echo).
type AnalyzeCommand struct {
	Text            string
	AnalysisJSON    string
	PolicyName      string
	EntityFilter    func(raw []byte) []byte // nil means no filter
}

// AnalyzeResult is the domain output before HTTP serialization.
type AnalyzeResult struct {
	RawJSON      []byte
	Minimization *policy.MinimizationReport
	PolicyReport *policy.PolicyReport
}

// Execute orchestrates minimization → engine → post-policy → RBAC filter.
func (a *Analyze) Execute(ctx context.Context, cmd AnalyzeCommand) (AnalyzeResult, error) {
	var zero AnalyzeResult
	text := strings.TrimSpace(cmd.Text)
	if text == "" {
		return zero, domain.ErrEmptyText
	}
	pname := strings.TrimSpace(cmd.PolicyName)
	ctx0, cancel0 := context.WithTimeout(ctx, a.Timeout)
	text, minRep, err := a.Policy.BeforeAnalyze(ctx0, pname, text)
	cancel0()
	if err != nil {
		return zero, errors.Join(ErrPolicyPhaseBefore, err)
	}
	ctxEng, cancelEng := context.WithTimeout(ctx, a.Timeout)
	defer cancelEng()
	var out string
	err = a.Circuit.Run(func() error {
		return a.Pool.With(ctxEng, func(ctx context.Context, eng AnalysisEngine) error {
			var e error
			out, e = eng.Analyze(ctx, text, cmd.AnalysisJSON)
			return e
		})
	})
	if err != nil {
		return zero, err
	}
	if !json.Valid([]byte(out)) {
		return zero, errors.New("moteur: réponse JSON invalide")
	}
	raw := []byte(out)
	raw, pr, err := a.Policy.AfterAnalyze(ctxEng, pname, raw)
	if err != nil {
		return zero, errors.Join(ErrPolicyPhaseAfter, err)
	}
	if cmd.EntityFilter != nil {
		raw = cmd.EntityFilter(raw)
	}
	return AnalyzeResult{
		RawJSON:      raw,
		Minimization: minRep,
		PolicyReport: pr,
	}, nil
}
