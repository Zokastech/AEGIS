# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Variables du module AWS — ECS Fargate, ALB + WAF, données managées, VPC privé.

# ---------------------------------------------------------------------------
# Général
# ---------------------------------------------------------------------------

variable "project_name" {
  type        = string
  description = "Préfixe logique des ressources (ex. aegis-prod)."
}

variable "environment" {
  type        = string
  description = "Nom d’environnement (dev, staging, prod)."
}

variable "aws_region" {
  type        = string
  description = "Région AWS principale (ex. eu-west-3 pour Paris)."
  default     = "eu-west-3"
}

variable "default_tags" {
  type        = map(string)
  description = "Tags additionnels fusionnés avec les tags par défaut du provider."
  default     = {}
}

# ---------------------------------------------------------------------------
# Réseau (VPC module)
# ---------------------------------------------------------------------------

variable "vpc_cidr" {
  type        = string
  description = "CIDR du VPC AEGIS."
  default     = "10.42.0.0/16"
}

variable "az_count" {
  type        = number
  description = "Nombre de zones de disponibilité (2–3 recommandé en production)."
  default     = 3
}

variable "enable_nat_gateway" {
  type        = bool
  description = "Active les NAT Gateway pour les sous-réseaux privés (ECS, accès Internet sortant)."
  default     = true
}

variable "single_nat_gateway" {
  type        = bool
  description = "Si true : un seul NAT (coût réduit, moins de résilience)."
  default     = false
}

# ---------------------------------------------------------------------------
# TLS (ACM)
# ---------------------------------------------------------------------------

variable "acm_domain_name" {
  type        = string
  description = "FQDN du certificat ACM servi par l’ALB (ex. api.aegis.example.com). Laisser vide pour désactiver la création du certificat."
  default     = ""
}

variable "acm_subject_alternative_names" {
  type        = list(string)
  description = "SAN additionnels pour le certificat ACM."
  default     = []
}

variable "create_route53_records" {
  type        = bool
  description = "Si true et zone_id renseigné : enregistrements DNS de validation ACM automatiques."
  default     = false
}

variable "route53_zone_id" {
  type        = string
  description = "ID de zone Route 53 hébergeant le domaine ACM (optionnel)."
  default     = ""
}

# ---------------------------------------------------------------------------
# ECS — images et capacité
# ---------------------------------------------------------------------------

variable "ecs_gateway_image" {
  type        = string
  description = "Image du conteneur aegis-gateway (URI ECR ou GHCR)."
  default     = "ghcr.io/zokastech/aegis-gateway:latest"
}

variable "ecs_core_image" {
  type        = string
  description = "Image du conteneur aegis-core."
  default     = "ghcr.io/zokastech/aegis-core:latest"
}

variable "ecs_dashboard_image" {
  type        = string
  description = "Image du conteneur aegis-dashboard."
  default     = "ghcr.io/zokastech/aegis-dashboard:latest"
}

variable "gateway_container_port" {
  type        = number
  description = "Port HTTP exposé par le gateway dans le conteneur."
  default     = 8080
}

variable "gateway_desired_count" {
  type        = number
  description = "Nombre de tâches Fargate pour le service gateway."
  default     = 2
}

variable "core_desired_count" {
  type        = number
  description = "Nombre de tâches Fargate pour aegis-core."
  default     = 1
}

variable "dashboard_desired_count" {
  type        = number
  description = "Nombre de tâches Fargate pour le dashboard."
  default     = 1
}

variable "fargate_cpu" {
  type        = number
  description = "Unités CPU Fargate pour chaque service (256, 512, 1024, …)."
  default     = 512
}

variable "fargate_memory" {
  type        = number
  description = "Mémoire Fargate (MiB), doit correspondre à la grille AWS pour le CPU choisi."
  default     = 1024
}

variable "enable_dashboard_service" {
  type        = bool
  description = "Déploie le service ECS dashboard (sinon uniquement core + gateway)."
  default     = true
}

# ---------------------------------------------------------------------------
# RDS PostgreSQL
# ---------------------------------------------------------------------------

variable "rds_enabled" {
  type        = bool
  description = "Crée une instance RDS PostgreSQL dans les sous-réseaux isolés."
  default     = true
}

variable "rds_engine_version" {
  type        = string
  description = "Version moteur PostgreSQL (ex. 16.3)."
  default     = "16.3"
}

variable "rds_instance_class" {
  type        = string
  description = "Classe d’instance RDS (ex. db.t4g.medium)."
  default     = "db.t4g.small"
}

variable "rds_allocated_storage" {
  type        = number
  description = "Stockage alloué (GiB)."
  default     = 20
}

variable "rds_database_name" {
  type        = string
  description = "Nom de la base créée à l’init."
  default     = "aegis"
}

variable "rds_username" {
  type        = string
  description = "Utilisateur maître RDS."
  default     = "aegis"
}

variable "rds_multi_az" {
  type        = bool
  description = "Haute disponibilité multi-AZ."
  default     = false
}

variable "rds_backup_retention_period" {
  type        = number
  description = "Rétention des snapshots automatiques (jours)."
  default     = 7
}

# ---------------------------------------------------------------------------
# ElastiCache Redis
# ---------------------------------------------------------------------------

variable "redis_enabled" {
  type        = bool
  description = "Crée un cluster ElastiCache Redis (réplication possible selon variables)."
  default     = true
}

variable "redis_node_type" {
  type        = string
  description = "Type de nœud ElastiCache (ex. cache.t4g.small)."
  default     = "cache.t4g.small"
}

variable "redis_num_cache_nodes" {
  type        = number
  description = "Nombre de nœuds (1 = standalone, >1 avec réplication selon moteur)."
  default     = 1
}

variable "redis_engine_version" {
  type        = string
  description = "Version Redis ElastiCache."
  default     = "7.1"
}

# ---------------------------------------------------------------------------
# ALB / WAF
# ---------------------------------------------------------------------------

variable "waf_enabled" {
  type        = bool
  description = "Associe un Web ACL WAFv2 régional à l’ALB."
  default     = true
}

variable "alb_deletion_protection" {
  type        = bool
  description = "Protection contre la suppression accidentelle de l’ALB."
  default     = true
}

variable "cloudwatch_log_retention_days" {
  type        = number
  description = "Rétention des journaux CloudWatch Logs pour les tâches ECS."
  default     = 30
}
