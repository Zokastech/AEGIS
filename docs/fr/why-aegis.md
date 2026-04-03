# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Pourquoi AEGIS — paysage concurrentiel

Cette page explique **comment AEGIS se positionne** par rapport aux alternatives courantes. Elle s’adresse aux architectes, acheteurs et partenaires qui évaluent la **détection et l’anonymisation de données personnelles** dans un contexte **réglementé européen**.

!!! note "Marché en évolution"
    Les noms de produits et les offres changent. Les arguments ci-dessous portent sur des **modèles d’architecture et de go-to-market**, pas sur un instantané fournisseur unique.

## Concurrents majeurs

### 1. Microsoft Presidio (l’incumbent)

| | |
|---|---|
| **Forces** | Maturité ; usage en production chez Microsoft ; écosystème Python large ; notoriété. |
| **Faiblesses** | Forte orientation **US / anglais** ; le RGPD n’est pas un axe de conception natif ; stacks **Python** plus difficiles à faire tenir en **très gros volume** avec des SLO de latence stricts. |
| **Gain AEGIS** | Conception **EU-first**, **multi-langue EU**, performances **Rust / Go**, **open source** **on-prem** sans dépendance runtime propriétaire. |

### 2. Outils de sécurité autour d’Elastic / conteneurs (ex. sécurité des charges de travail)

Certaines équipes combinent l’**Elastic Stack** avec des produits de **sécurité conteneur / workload**. Ces outils excellent sur la **télémétrie infra et runtime**, pas sur la détection et l’anonymisation **centrées PII** dans du **texte libre**.

| | |
|---|---|
| **Forces** | Intégration forte aux workflows **SOC** et logging. |
| **Faiblesses** | **Pas centrés PII** ; orientation **sécurité plateforme** plutôt que **privacy données** (parcours DPIA, pseudonymisation réversible, traces d’audit). |
| **Gain AEGIS** | **Focus PII pur** : détection + **anonymisation** pilotée par politiques, architecture orientée **privacy**. |

### 3. Startups propriétaires « UI moderne » (type Stanza)

| | |
|---|---|
| **Forces** | UI soignée ; discours **LLM** ; traction auprès de startups **US**. |
| **Faiblesses** | **Propriétaire** ; souvent **SaaS only** ; posture **US-first** ; coût qui grimpe avec l’usage. |
| **Gain AEGIS** | **Open source** ; **self-hosted** ; cadrage **RGPD natif** ; stack **transparente** ; **extensible** (recognizers, opérateurs). |

### 4. AWS Macie

| | |
|---|---|
| **Forces** | Écosystème **AWS** ; mise à l’échelle managée massive. |
| **Faiblesses** | **Cloud-only** (en pratique) ; **propriétaire** ; pas une spécialité **texte multi-langue EU** ; coût et **maîtrise** pour les tenants réglementés. |
| **Gain AEGIS** | **On-prem** et **résidence des données UE** ; **open source** ; **coût maîtrisé** quand vous opérez la stack. |

## Vos avantages concurrentiels clés

| Dimension | AEGIS | Pourquoi c’est important |
|-----------|-------|--------------------------|
| **EU-first** | Conception **RGPD** dès le départ, pas en rattrapage | Les entreprises EU veulent de la **confiance locale**, pas une dépendance **US-cloud** systématique. |
| **Open source** | **Apache 2.0 / MIT** ; la communauté peut auditer et étendre | Face au propriétaire (Stanza, AWS, etc.) : **transparence**, **adoption**, **talent**. |
| **Multi-langue EU** | Roadmap **30+ langues** (pas seulement l’anglais) | Peu de concurrents le font bien — enjeu majeur pour la **conformité multi-pays**. |
| **Performance** | **Rust + Go**, cibles **sub-100 ms** pour beaucoup de déploiements, **on-prem** | Python (Presidio) = limites volume/latence ; SaaS = **latence** + **dépendance**. |
| **Déterministe** | **FPE**, **pseudonymisation** reproductible | Par rapport au pur ML qui **dérive** — **pistes d’audit** plus propres. |
| **Écosystème (roadmap)** | **Kafka**, **Spark**, **dbt**, **Airflow**… | Aligné sur les workflows **data engineering** et **analytics** modernes. |

## Où vous êtes vulnérables (et comment répondre)

| Écart | Défi | Riposte |
|-------|------|---------|
| **Maturité** | Presidio = **battle-tested** Microsoft ; AEGIS = plus jeune | **Benchmarks** précision/rappel ; **success stories** early adopters. |
| **Taille de communauté** | Presidio = plus grande **mindshare** | Construire la **communauté** tôt : confs privacy EU, **ambassadeurs**, doc claire. |
| **Sales / GTM** | AWS / Stanza = **forces de vente** | **Partenaires** (ESN, cabinets privacy) + croissance **product-led** (offre généreuse startups / associations). |
| **Intégrations** | « Roadmap » vs **livré** | Prioriser **une** intégration clé (ex. Kafka *ou* Spark) et la **livrer solidement**. Mieux vaut **1** intégration mature que **5** bêtas. |

## Opportunité stratégique : le « Presidio européen »

Plusieurs facteurs structurels favorisent un moteur **européen**, **ouvert** et **centré PII** :

- **Microsoft** ne pivotera pas Presidio en **RGPD-first** (legacy, gravité produit US).
- **Macie** restera **cloud propriétaire** pour le cœur de l’offre.
- Beaucoup d’offres **US** manquent d’**ancrage EU** crédible sur la souveraineté.

**AEGIS** vise à être l’option **EU-native**, **self-hostable** et **transparente** pour la **détection et l’anonymisation de PII** dans le **texte**.

### En tirer profit

1. **RFP EU** sur **conformité RGPD + open source + on-prem / résidence UE**.
2. **Partenaires EU** : intégrateurs (Capgemini, Accenture, mid-market), **cabinets conseil privacy**.
3. **GTM pricing** : **gratuit durable** (ou très bas) pour **startups / associations** → adoption → **upsell enterprise**.
4. **Community first** : sponsoriser confs **RGPD / privacy**, documentation **FR / DE / NL**, **devrel** fort.
5. **Blitz benchmarks** : publier **≥95 %** précision vs Presidio sur jeux de données **convenus**, avec **caveats** honnêtes.

## Voir aussi

- [Migration depuis Presidio](migration-presidio.md) — mapping technique
- [Architecture](architecture.md) — pipeline et composants
- [Sécurité — alignement RGPD](security/rgpd-compliance.md)
