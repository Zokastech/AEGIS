# AEGIS — zokastech.fr — Apache 2.0 / MIT

Répertoire des **modules Terraform** pour déployer AEGIS sur plusieurs clouds. Chaque sous-dossier est un **module racine** autonome (variables, outputs, `README`).

| Dossier | Plateforme | Composants clés |
|---------|------------|-----------------|
| [`aws/`](aws/) | Amazon Web Services | VPC (public / privé / données), ECS Fargate, ALB, WAFv2, ACM, RDS PostgreSQL, ElastiCache Redis, CloudWatch Logs |
| [`gcp/`](gcp/) | Google Cloud | VPC, Serverless VPC Access, Cloud Run v2, Cloud SQL (IP privée), Memorystore Redis, Secret Manager, firewall de base |
| [`azure/`](azure/) | Microsoft Azure | VNet, NSG, Container Apps Environment, Container Apps, PostgreSQL Flexible (privé), Redis Standard (TLS, endpoint public — voir README), Log Analytics |
| [`ovh/`](ovh/) | OVHcloud (UE) | Managed Kubernetes, Object Storage S3-compatible pour modèles, documentation souveraineté |

## Licence

Apache-2.0 et MIT — **AEGIS** par [zokastech.fr](https://zokastech.fr).
