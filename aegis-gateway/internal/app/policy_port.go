// AEGIS — zokastech.fr — Apache 2.0 / MIT

package app

import (
	"context"

	"github.com/zokastech/aegis/aegis-gateway/policy"
)

// PolicyGate is the compliance-policy port (implementation: policy.PolicyEngine).
type PolicyGate interface {
	BeforeAnalyze(ctx context.Context, policyName, text string) (string, *policy.MinimizationReport, error)
	AfterAnalyze(ctx context.Context, policyName string, raw []byte) ([]byte, *policy.PolicyReport, error)
}

// PolicyEngineGate adapte *policy.PolicyEngine au port PolicyGate.
type PolicyEngineGate struct {
	Eng *policy.PolicyEngine
}

func (g PolicyEngineGate) BeforeAnalyze(ctx context.Context, policyName, text string) (string, *policy.MinimizationReport, error) {
	if g.Eng == nil || policyName == "" {
		return text, nil, nil
	}
	pol, err := g.Eng.Policy(policyName)
	if err != nil {
		return "", nil, err
	}
	return g.Eng.BeforeAnalyze(ctx, pol, text)
}

func (g PolicyEngineGate) AfterAnalyze(ctx context.Context, policyName string, raw []byte) ([]byte, *policy.PolicyReport, error) {
	if g.Eng == nil || policyName == "" {
		return raw, nil, nil
	}
	pol, err := g.Eng.Policy(policyName)
	if err != nil {
		return nil, nil, err
	}
	return g.Eng.AfterAnalyze(ctx, pol, raw)
}
