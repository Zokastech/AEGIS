# AEGIS — zokastech.fr — Apache 2.0 / MIT

variable "ovh_project_id" {
  type        = string
  description = "Identifiant du projet Public Cloud OVHcloud (service name pour l’API kube)."
}

variable "environment" {
  type        = string
  description = "Environnement (dev, staging, prod)."
}

variable "cluster_name" {
  type        = string
  description = "Nom du cluster Kubernetes managé."
  default     = "aegis"
}

variable "kube_region" {
  type        = string
  description = "Région OpenStack OVH pour le control plane (ex. GRA5, RBX-A, DE1 — privilégier l’UE)."
  default     = "GRA5"
}

variable "kubernetes_version" {
  type        = string
  description = "Version Kubernetes supportée par OVH pour la région (ex. 1.28)."
  default     = "1.28"
}

variable "nodepool_name" {
  type        = string
  description = "Nom du node pool worker."
  default     = "aegis-workers"
}

variable "node_flavor" {
  type        = string
  description = "Flavor des nœuds (ex. b3-8, b3-16 — voir catalogue OVH)."
  default     = "b3-8"
}

variable "node_desired" {
  type        = number
  description = "Nombre de nœuds souhaités dans le pool (ajustez min/max via API / console OVH si besoin)."
  default     = 3
}

variable "create_models_bucket" {
  type        = bool
  description = "Crée un bucket Object Storage (API S3 compatible OVH) pour les modèles ONNX / artefacts."
  default     = false
}

variable "models_bucket_name" {
  type        = string
  description = "Nom global du bucket (unique sur l’endpoint régional)."
  default     = ""
}

variable "object_storage_s3_endpoint" {
  type        = string
  description = "Endpoint S3 OVH pour la région (ex. https://s3.gra.io.cloud.ovh.net)."
  default     = "https://s3.gra.io.cloud.ovh.net"
}

variable "object_storage_s3_region" {
  type        = string
  description = "Région passée au client S3 (souvent le code région courte, ex. gra)."
  default     = "gra"
}

variable "object_storage_access_key" {
  type        = string
  description = "Access key utilisateur S3 (utilisateur OpenStack dédié recommandé)."
  default     = ""
  sensitive   = true
}

variable "object_storage_secret_key" {
  type        = string
  description = "Secret key utilisateur S3."
  default     = ""
  sensitive   = true
}

variable "tags" {
  type        = map(string)
  description = "Métadonnées / labels à propager sur les ressources OVH supportées."
  default     = {}
}
