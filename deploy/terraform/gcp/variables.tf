# AEGIS — zokastech.fr — Apache 2.0 / MIT

variable "project_id" {
  type        = string
  description = "ID du projet GCP (ex. my-aegis-prod)."
}

variable "region" {
  type        = string
  description = "Région principale (ex. europe-west9 pour Paris)."
  default     = "europe-west9"
}

variable "environment" {
  type        = string
  description = "Nom d’environnement (dev, staging, prod)."
}

variable "network_name" {
  type        = string
  description = "Nom du réseau VPC custom."
  default     = "aegis-vpc"
}

variable "vpc_cidr" {
  type        = string
  description = "CIDR du VPC."
  default     = "10.60.0.0/16"
}

variable "connector_cidr" {
  type        = string
  description = "Plage /28 pour le Serverless VPC Access Connector (non chevauchante)."
  default     = "10.60.240.0/28"
}

variable "sql_peering_prefix_length" {
  type        = number
  description = "Préfixe pour la plage réservée au peering Service Networking (Cloud SQL)."
  default     = 16
}

variable "gateway_image" {
  type        = string
  description = "Image Artifact Registry / GCR pour aegis-gateway."
  default     = "ghcr.io/zokastech/aegis-gateway:latest"
}

variable "core_image" {
  type        = string
  description = "Image pour aegis-core."
  default     = "ghcr.io/zokastech/aegis-core:latest"
}

variable "dashboard_image" {
  type        = string
  description = "Image pour aegis-dashboard."
  default     = "ghcr.io/zokastech/aegis-dashboard:latest"
}

variable "cloud_run_min_instances" {
  type        = number
  description = "Instances minimales Cloud Run (gateway)."
  default     = 0
}

variable "cloud_run_max_instances" {
  type        = number
  description = "Instances maximales Cloud Run (gateway)."
  default     = 10
}

variable "cloud_run_cpu" {
  type        = string
  description = "CPU alloué au conteneur gateway (ex. \"1\" ou \"2\")."
  default     = "1"
}

variable "cloud_run_memory" {
  type        = string
  description = "Mémoire (ex. \"512Mi\", \"1Gi\")."
  default     = "512Mi"
}

variable "enable_cloud_sql" {
  type        = bool
  description = "Crée une instance Cloud SQL PostgreSQL avec IP privée."
  default     = true
}

variable "sql_tier" {
  type        = string
  description = "Machine type Cloud SQL (ex. db-f1-micro en dev, db-custom-2-7680 en prod)."
  default     = "db-f1-micro"
}

variable "sql_disk_size" {
  type        = number
  description = "Taille disque données (Go)."
  default     = 10
}

variable "sql_database_name" {
  type        = string
  description = "Nom de la base applicative."
  default     = "aegis"
}

variable "sql_user" {
  type        = string
  description = "Utilisateur PostgreSQL."
  default     = "aegis"
}

variable "enable_memorystore" {
  type        = bool
  description = "Crée une instance Memorystore Redis."
  default     = true
}

variable "redis_memory_gb" {
  type        = number
  description = "Taille mémoire Redis (Go)."
  default     = 1
}

variable "redis_tier" {
  type        = string
  description = "BASIC ou STANDARD_HA."
  default     = "BASIC"
}

variable "allow_public_cloud_run" {
  type        = bool
  description = "Si true : IAM invoker allUsers sur le service gateway (tests uniquement)."
  default     = false
}

variable "default_labels" {
  type        = map(string)
  description = "Labels GCP fusionnés sur les ressources supportées."
  default     = {}
}
