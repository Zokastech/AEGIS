# AEGIS — zokastech.fr — Apache 2.0 / MIT

Module Terraform **AWS** pour héberger la stack AEGIS : **VPC** (public / privé / données isolées), **ECS Fargate** (gateway, core, dashboard), **ALB** avec **WAFv2** optionnel, **ACM**, **RDS PostgreSQL**, **ElastiCache Redis**, **CloudWatch Logs**.

## Architecture

| Couche | Ressources |
|--------|------------|
| Edge | ALB (HTTP → redirection HTTPS si certificat), WAF régional |
| Compute | ECS Fargate dans sous-réseaux privés (NAT pour images / APIs) |
| Données | RDS + Redis dans sous-réseaux « database » sans accès Internet direct |

Le **gateway** est enregistré sur l’ALB (health check `/health/ready`). Le **core** et le **dashboard** tournent en privé sans cible ALB par défaut (exposez-les via une 2ᵉ règle d’écoute ou un autre schéma selon vos besoins).

## Prérequis

- Terraform ≥ 1.5, AWS CLI configuré
- Quotas : VPC, ECS, ALB, RDS, ElastiCache selon les tailles choisies
- Images conteneurs accessibles depuis les tâches (ECR public, GHCR avec auth task execution si privé)

## Utilisation

```bash
cd deploy/terraform/aws
terraform init
terraform plan \
  -var="project_name=aegis" \
  -var="environment=prod" \
  -var="aws_region=eu-west-3" \
  -var="acm_domain_name=api.example.com" \
  -var="create_route53_records=true" \
  -var="route53_zone_id=Zxxxxxxxx"
terraform apply
```

### Certificat ACM

- Si `create_route53_records=true` et `route53_zone_id` est renseigné, la validation DNS est automatique et le listener **HTTPS** peut s’attacher au certificat émis.
- Sinon, créez les enregistrements CNAME affichés dans la console ACM puis ré-appliquez une fois le certificat **Issued**.

### Variables sensibles

Le mot de passe maître RDS est géré par AWS (**Secrets Manager**). Le rôle d’exécution ECS peut lire ce secret ; le gateway reçoit `AEGIS_DB_PASSWORD` via la référence ECS. Adaptez les variables d’environnement côté application si vous utilisez un seul `DATABASE_URL` (ex. construire l’URL dans le code ou via un secret dédié).

## Outputs principaux

Voir `outputs.tf` : `alb_dns_name`, `vpc_id`, `rds_endpoint`, `redis_primary_endpoint`, `cloudwatch_log_groups`, `waf_web_acl_arn`, etc.

## Conformité et durcissement

- Stockage RDS chiffré, sous-réseaux dédiés aux bases.
- WAF : ensembles de règles managés (CommonRuleSet, KnownBadInputs) — à compléter (géo, rate limit, logs S3/Kinesis) selon votre politique de sécurité.
- Ajustez `skip_final_snapshot` / `deletion_protection` pour la production (déjà liés à `environment`).

## Licence

Apache-2.0 et MIT — **AEGIS** par [zokastech.fr](https://zokastech.fr).
