# AEGIS — zokastech.fr — Apache 2.0 / MIT

resource "azurerm_container_app_environment" "main" {
  name                           = "${local.name}-cae"
  location                       = var.location
  resource_group_name            = var.resource_group_name
  log_analytics_workspace_id     = azurerm_log_analytics_workspace.main.id
  infrastructure_subnet_id       = azurerm_subnet.containerapps.id
  internal_load_balancer_enabled = false
  tags                           = local.tags

  depends_on = [azurerm_subnet_network_security_group_association.containerapps]
}

resource "azurerm_container_app" "gateway" {
  name                         = "${local.name}-gw"
  container_app_environment_id = azurerm_container_app_environment.main.id
  resource_group_name          = var.resource_group_name
  revision_mode                = "Single"

  dynamic "secret" {
    for_each = var.enable_postgresql ? [1] : []
    content {
      name  = "db-password"
      value = random_password.postgres[0].result
    }
  }

  dynamic "secret" {
    for_each = var.enable_redis ? [1] : []
    content {
      name  = "redis-key"
      value = azurerm_redis_cache.main[0].primary_access_key
    }
  }

  template {
    min_replicas = 1
    max_replicas = 10

    container {
      name   = "gateway"
      image  = var.gateway_image
      cpu    = 0.5
      memory = "1Gi"

      env {
        name  = "AEGIS_POLICY"
        value = "/policies/gdpr-strict.yaml"
      }

      dynamic "env" {
        for_each = var.enable_postgresql ? {
          AEGIS_DB_HOST = azurerm_postgresql_flexible_server.main[0].fqdn
          AEGIS_DB_PORT = "5432"
          AEGIS_DB_NAME = var.postgresql_database_name
          AEGIS_DB_USER = var.postgresql_admin_user
        } : {}
        content {
          name  = env.key
          value = env.value
        }
      }

      dynamic "env" {
        for_each = var.enable_postgresql ? [1] : []
        content {
          name        = "AEGIS_DB_PASSWORD"
          secret_name = "db-password"
        }
      }

      dynamic "env" {
        for_each = var.enable_redis ? [1] : []
        content {
          name  = "REDIS_ADDR"
          value = "${azurerm_redis_cache.main[0].hostname}:6380"
        }
      }

      dynamic "env" {
        for_each = var.enable_redis ? [1] : []
        content {
          name        = "REDIS_PASSWORD"
          secret_name = "redis-key"
        }
      }
    }
  }

  ingress {
    external_enabled = true
    target_port      = 8080
    transport        = "auto"

    traffic_weight {
      percentage      = 100
      latest_revision = true
    }
  }

  tags = local.tags
}

resource "azurerm_container_app" "core" {
  name                         = "${local.name}-core"
  container_app_environment_id = azurerm_container_app_environment.main.id
  resource_group_name          = var.resource_group_name
  revision_mode                = "Single"

  template {
    min_replicas = 0
    max_replicas = 5

    container {
      name    = "core"
      image   = var.core_image
      cpu     = 0.25
      memory  = "0.5Gi"
      command = ["/bin/sh", "-c", "sleep infinity"]
    }
  }

  ingress {
    external_enabled = false
    target_port      = 8080
    traffic_weight {
      percentage      = 100
      latest_revision = true
    }
  }

  tags = local.tags
}

resource "azurerm_container_app" "dashboard" {
  name                         = "${local.name}-dash"
  container_app_environment_id = azurerm_container_app_environment.main.id
  resource_group_name          = var.resource_group_name
  revision_mode                = "Single"

  template {
    min_replicas = 0
    max_replicas = 5

    container {
      name   = "dashboard"
      image  = var.dashboard_image
      cpu    = 0.25
      memory = "0.5Gi"
    }
  }

  ingress {
    external_enabled = false
    target_port      = 8080
    traffic_weight {
      percentage      = 100
      latest_revision = true
    }
  }

  tags = local.tags
}
