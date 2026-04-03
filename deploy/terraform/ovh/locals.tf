# AEGIS — zokastech.fr — Apache 2.0 / MIT

locals {
  name = "${var.cluster_name}-${var.environment}"

  default_bucket_name = var.models_bucket_name != "" ? var.models_bucket_name : "${local.name}-models"

  object_storage_ready = var.create_models_bucket && var.object_storage_access_key != "" && var.object_storage_secret_key != ""
}
