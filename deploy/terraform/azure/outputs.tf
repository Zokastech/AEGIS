# AEGIS — zokastech.fr — Apache 2.0 / MIT

output "virtual_network_id" {
  description = "ID du VNet AEGIS."
  value       = azurerm_virtual_network.main.id
}

output "container_app_environment_id" {
  description = "ID de l’environnement Container Apps."
  value       = azurerm_container_app_environment.main.id
}

output "gateway_fqdn" {
  description = "FQDN public du gateway (Container App)."
  value       = azurerm_container_app.gateway.latest_revision_fqdn
}

output "postgresql_fqdn" {
  description = "FQDN privé PostgreSQL Flexible Server (si activé)."
  value       = try(azurerm_postgresql_flexible_server.main[0].fqdn, null)
}

output "redis_hostname" {
  description = "Hôte Redis (Premium VNet, si activé)."
  value       = try(azurerm_redis_cache.main[0].hostname, null)
}

output "log_analytics_workspace_id" {
  description = "ID de l’espace de travail Log Analytics (logs Container Apps)."
  value       = azurerm_log_analytics_workspace.main.id
}
