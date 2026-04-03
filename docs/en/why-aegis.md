# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Why AEGIS — competitive landscape

This page explains **how AEGIS is positioned** against common alternatives. It is meant for architects, buyers, and partners evaluating PII detection and anonymization in **EU-regulated** environments.

!!! note "Evolving market"
    Product names and packaging change. The points below focus on **architectural and go-to-market patterns**, not on a single vendor snapshot.

## Major alternatives

### 1. Microsoft Presidio (incumbent)

| | |
|---|---|
| **Strengths** | Mature; used in production at Microsoft; large Python ecosystem; broad name recognition. |
| **Limitations** | Strongest fit for **US / English** workloads; GDPR is not a first-class design axis; **Python** stacks are harder to run at **high volume** with strict latency SLOs. |
| **Where AEGIS wins** | **EU-first** design, **multi-language EU** roadmap, **Rust / Go** performance, **open source** you can run **on-prem** without vendor runtime lock-in. |

### 2. Elastic Stack–adjacent security tooling (e.g. container / workload security)

Some teams combine the **Elastic Stack** with **container or workload security** products. Those tools excel at **infra and runtime telemetry**, not at **PII-centric** detection and anonymization in **free text**.

| | |
|---|---|
| **Strengths** | Deep integration with logging and security operations workflows. |
| **Limitations** | **Not PII-centric**; oriented toward **platform security** rather than **data privacy** use cases (DPIA-friendly flows, reversible pseudonymization, audit-friendly traces). |
| **Where AEGIS wins** | **Pure PII focus**: detection + policy-driven **anonymization**, privacy-oriented **architecture**. |

### 3. Proprietary “modern UI” startups (e.g. Stanza-class offerings)

| | |
|---|---|
| **Strengths** | Polished UIs; **LLM**-related messaging; traction with **US** startups. |
| **Limitations** | **Proprietary**; often **SaaS-only**; **US-first** posture; cost scales with seats / volume. |
| **Where AEGIS wins** | **Open source**; **self-hosted**; **GDPR-native** framing; **transparent** stack; **extensible** recognizers and operators. |

### 4. AWS Macie

| | |
|---|---|
| **Strengths** | **AWS** ecosystem; massive managed scale. |
| **Limitations** | **Cloud-only** (for practical purposes); **proprietary**; not a **multi-language EU text** specialty; cost and **control** trade-offs for regulated tenants. |
| **Where AEGIS wins** | **On-prem** and **EU data residency**; **open source**; **cost-effective** at scale when you own the stack. |

## Key differentiation (at a glance)

| Dimension | AEGIS | Why it matters |
|-----------|-------|----------------|
| **EU-first** | GDPR-oriented design from the start, not a retrofit | EU buyers want **local trust** and reduced **US-cloud dependency** for sensitive text. |
| **Open source** | Apache 2.0 / MIT; community can inspect and extend | Versus proprietary SaaS: **transparency**, easier **adoption**, stronger **talent** attraction. |
| **Multi-language EU** | Roadmap toward **30+** languages—not “English plus extras” | Rarely done well by incumbents; critical for **multi-country compliance**. |
| **Performance** | **Rust + Go**; sub-100 ms class targets for many deployments; **on-prem** | Python-only stacks can struggle at volume; SaaS adds **latency** and **dependency**. |
| **Deterministic transforms** | **Format-preserving encryption**, reproducible **pseudonymization** | Cleaner **audit trails** than “ML-only” approaches that drift. |
| **Ecosystem roadmap** | **Kafka**, **Spark**, **dbt**, **Airflow**-style integration patterns | Matches modern **data engineering** and **analytics** workflows. |

## Where AEGIS is comparatively weaker (and how to address it)

| Gap | Challenge | Practical response |
|-----|-----------|-------------------|
| **Maturity vs Presidio** | Presidio is **battle-tested** at large scale; AEGIS is earlier. | Publish **precision / recall** benchmarks on public corpora; collect **early adopter** case studies. |
| **Community size** | Presidio has larger **mindshare** today. | Invest early in **community**: talks, integrations, ambassadors, clear docs. |
| **Sales / GTM** | Hyperscalers and SaaS vendors have **enterprise sales** machines. | **Partner channel** (SI / privacy consultancies) + **product-led** motion (generous tier for startups / nonprofits). |
| **Integrations: roadmap vs shipped** | Buyers compare **today’s** connectors. | **Ship one integration deeply** (e.g. Kafka *or* Spark) rather than five shallow betas. |

## Strategic opportunity: an EU-native “Presidio-class” play

Several structural factors favor a **European**, **open**, **PII-first** engine:

- **Microsoft** is unlikely to reposition Presidio as **GDPR-first** (legacy stack, US-centric product gravity).
- **AWS Macie** will remain **cloud-proprietary** for the core value proposition.
- Many **US-born** SaaS tools lack a credible **EU anchor** for sovereignty narratives.

**AEGIS** aims to be the **EU-native**, **self-hostable**, **transparent** option for teams that need **PII detection and anonymization** in **text**—not just security telemetry.

### Ways to capitalize

1. **RFPs** that bundle **GDPR compliance** + **open source** + **on-prem / EU residency**.
2. **EU systems integrators and privacy consultancies** as implementation partners.
3. **Pricing-led GTM**: a **free** or low-cost tier for **startups / nonprofits** to drive adoption, with **enterprise** upsell for support and hardening.
4. **Community-first**: sponsor **GDPR / privacy** events; strong **developer relations**; docs in **FR / DE / NL** (and more) alongside English.
5. **Benchmark blitz**: when metrics are solid, publish **≥95%** style accuracy comparisons vs Presidio on **agreed** datasets (with honest caveats).

## See also

- [Migration from Presidio](migration-presidio.md) — technical mapping
- [Architecture](architecture.md) — pipeline and components
- [Security — GDPR alignment](security/rgpd-compliance.md)
