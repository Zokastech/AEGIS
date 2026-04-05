# AEGIS — zokastech.fr — Apache 2.0 / MIT

provider "ovh" {
  endpoint = "ovh-eu"
}

provider "aws" {
  alias = "ovh_os"

  region                      = var.object_storage_s3_region
  access_key                  = var.object_storage_access_key
  secret_key                  = var.object_storage_secret_key
  skip_credentials_validation = true
  skip_requesting_account_id  = true
  skip_metadata_api_check     = true
  skip_region_validation      = true

  endpoints {
    s3 = var.object_storage_s3_endpoint
  }
}
