# AEGIS — zokastech.fr — Apache 2.0 / MIT

resource "google_cloud_run_v2_service" "gateway" {
  name     = replace("${local.name}-gw", "_", "-")
  location = var.region
  project  = var.project_id

  labels = local.labels

  template {
    service_account = google_service_account.run.email

    scaling {
      min_instance_count = var.cloud_run_min_instances
      max_instance_count = var.cloud_run_max_instances
    }

    vpc_access {
      connector = google_vpc_access_connector.main.id
      egress    = "PRIVATE_RANGES_ONLY"
    }

    max_instance_request_concurrency = 80

    containers {
      image = var.gateway_image

      ports {
        container_port = 8080
      }

      resources {
        limits = {
          cpu    = var.cloud_run_cpu
          memory = var.cloud_run_memory
        }
      }

      dynamic "env" {
        for_each = var.enable_memorystore ? { REDIS_ADDR = "${google_redis_instance.main[0].host}:${google_redis_instance.main[0].port}" } : {}
        content {
          name  = env.key
          value = env.value
        }
      }

      dynamic "env" {
        for_each = var.enable_cloud_sql ? {
          AEGIS_DB_HOST = google_sql_database_instance.main[0].private_ip_address
          AEGIS_DB_PORT = "5432"
          AEGIS_DB_NAME = var.sql_database_name
          AEGIS_DB_USER = var.sql_user
        } : {}
        content {
          name  = env.key
          value = env.value
        }
      }

      dynamic "env" {
        for_each = var.enable_cloud_sql ? [1] : []
        content {
          name = "AEGIS_DB_PASSWORD"
          value_source {
            secret_key_ref {
              secret  = google_secret_manager_secret.sql_password[0].secret_id
              version = "latest"
            }
          }
        }
      }

      env {
        name  = "AEGIS_POLICY"
        value = "/policies/gdpr-strict.yaml"
      }
    }
  }

  ingress = "INGRESS_TRAFFIC_ALL"

  depends_on = [google_vpc_access_connector.main]
}

resource "google_cloud_run_v2_service_iam_member" "gateway_public" {
  count = var.allow_public_cloud_run ? 1 : 0

  project  = var.project_id
  location = var.region
  name     = google_cloud_run_v2_service.gateway.name
  role     = "roles/run.invoker"
  member   = "allUsers"
}

resource "google_cloud_run_v2_service" "core" {
  name     = replace("${local.name}-core", "_", "-")
  location = var.region
  project  = var.project_id

  labels = local.labels

  template {
    service_account = google_service_account.run.email

    scaling {
      min_instance_count = 0
      max_instance_count = 5
    }

    vpc_access {
      connector = google_vpc_access_connector.main.id
      egress    = "PRIVATE_RANGES_ONLY"
    }

    containers {
      image   = var.core_image
      command = ["sleep"]
      args    = ["infinity"]
    }
  }

  ingress = "INGRESS_TRAFFIC_INTERNAL_ONLY"

  depends_on = [google_vpc_access_connector.main]
}

resource "google_cloud_run_v2_service" "dashboard" {
  name     = replace("${local.name}-dash", "_", "-")
  location = var.region
  project  = var.project_id

  labels = local.labels

  template {
    service_account = google_service_account.run.email

    scaling {
      min_instance_count = 0
      max_instance_count = 5
    }

    vpc_access {
      connector = google_vpc_access_connector.main.id
      egress    = "PRIVATE_RANGES_ONLY"
    }

    containers {
      image = var.dashboard_image
      ports {
        container_port = 8080
      }
    }
  }

  ingress = "INGRESS_TRAFFIC_INTERNAL_ONLY"

  depends_on = [google_vpc_access_connector.main]
}
