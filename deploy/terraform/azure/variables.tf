# AEGIS — zokastech.fr — Apache 2.0 / MIT

variable "resource_group_name" {
  type        = string
  description = "Nom du groupe de ressources (créé si create_resource_group)."
}

variable "location" {
  type        = string
  description = "Région Azure (ex. France Central pour résidence UE)."
  default     = "francecentral"
}

variable "create_resource_group" {
  type        = bool
  description = "Si true : crée le groupe de ressources."
  default     = true
}

variable "environment" {
  type        = string
  description = "Environnement logique (dev, staging, prod)."
}

variable "vnet_address_space" {
  type        = list(string)
  description = "Espace d’adressage du VNet."
  default     = ["10.70.0.0/16"]
}

variable "containerapps_subnet_cidr" {
  type        = string
  description = "Sous-réseau délégué Container Apps (minimum /23 recommandé)."
  default     = "10.70.0.0/23"
}

variable "database_subnet_cidr" {
  type        = string
  description = "Sous-réseau privé pour PostgreSQL Flexible Server."
  default     = "10.70.8.0/24"
}

variable "gateway_image" {
  type        = string
  description = "Image conteneur aegis-gateway (ACR ou registre public)."
  default     = "ghcr.io/zokastech/aegis-gateway:latest"
}

variable "core_image" {
  type        = string
  description = "Image aegis-core."
  default     = "ghcr.io/zokastech/aegis-core:latest"
}

variable "dashboard_image" {
  type        = string
  description = "Image aegis-dashboard."
  default     = "ghcr.io/zokastech/aegis-dashboard:latest"
}

variable "enable_postgresql" {
  type        = bool
  description = "Déploie Azure Database for PostgreSQL Flexible Server."
  default     = true
}

variable "postgresql_sku" {
  type        = string
  description = "SKU Flexible Server (ex. B_Standard_B1ms, GP_Standard_D2s_v3)."
  default     = "B_Standard_B1ms"
}

variable "postgresql_storage_mb" {
  type        = number
  description = "Stockage alloué (Mo)."
  default     = 32768
}

variable "postgresql_database_name" {
  type        = string
  description = "Nom de la base applicative."
  default     = "aegis"
}

variable "postgresql_admin_user" {
  type        = string
  description = "Identifiant administrateur PostgreSQL."
  default     = "aegis"
}

variable "enable_redis" {
  type        = bool
  description = "Déploie Azure Cache for Redis (SKU Standard, endpoint public TLS — voir README pour private endpoint / Enterprise)."
  default     = false
}

variable "redis_capacity" {
  type        = number
  description = "Capacité du cache (dépend de la famille / SKU)."
  default     = 1
}

variable "redis_family" {
  type        = string
  description = "Famille Redis : C pour Basic/Standard (recommandé pour ce module)."
  default     = "C"
}

variable "redis_sku_name" {
  type        = string
  description = "SKU (Standard : endpoint public, chiffrement TLS obligatoire côté client)."
  default     = "Standard"
}

variable "tags" {
  type        = map(string)
  description = "Tags Azure."
  default     = {}
}
