# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Security

Security documentation for AEGIS covers **deployment hardening**, **threat modeling**, **GDPR-oriented technical measures**, and **supply-chain artifacts** (SBOM, signatures).

| Document | Description |
|----------|-------------|
| [Third-party licenses](../../../THIRD_PARTY_LICENSES.md) | Stacks (Rust, Go, npm, Python, Docker), Prometheus/Grafana, inventory commands |
| [Hardening](hardening.md) | Production checklist, network policies, secret rotation |
| [Threat model (STRIDE)](threat-model.md) | Data-flow diagram, STRIDE analysis, risk matrix (DPO-friendly) |
| [GDPR alignment](rgpd-compliance.md) | Technical measures mapped to GDPR articles (non-legal summary) |

Vulnerability reporting: see [`SECURITY.md` on GitHub](https://github.com/zokastech/aegis/blob/main/SECURITY.md).

SBOM generation: `scripts/generate-sbom.sh` — described in `SECURITY.md`.
