# AEGIS — zokastech.fr — Apache 2.0 / MIT

resource "random_password" "sql" {
  count   = var.enable_cloud_sql ? 1 : 0
  length  = 24
  special = false
}

resource "google_sql_database_instance" "main" {
  count            = var.enable_cloud_sql ? 1 : 0
  name             = replace("${local.name}-pg", "_", "-")
  database_version = "POSTGRES_16"
  region           = var.region

  settings {
    tier              = var.sql_tier
    disk_size         = var.sql_disk_size
    disk_autoresize   = true
    availability_type = "ZONAL"

    ip_configuration {
      ipv4_enabled                                  = false
      private_network                               = google_compute_network.main.id
      enable_private_path_for_google_cloud_services = true
    }

    backup_configuration {
      enabled = var.environment == "prod"
    }

    deletion_protection_enabled = var.environment == "prod"
  }

  depends_on = [google_service_networking_connection.private]
}

resource "google_sql_database" "app" {
  count    = var.enable_cloud_sql ? 1 : 0
  name     = var.sql_database_name
  instance = google_sql_database_instance.main[0].name
}

resource "google_sql_user" "app" {
  count    = var.enable_cloud_sql ? 1 : 0
  name     = var.sql_user
  instance = google_sql_database_instance.main[0].name
  password = random_password.sql[0].result
}

resource "google_secret_manager_secret" "sql_password" {
  count     = var.enable_cloud_sql ? 1 : 0
  secret_id = replace("${local.name}-sql-pwd", "_", "-")
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "sql_password" {
  count       = var.enable_cloud_sql ? 1 : 0
  secret      = google_secret_manager_secret.sql_password[0].id
  secret_data = random_password.sql[0].result
}
