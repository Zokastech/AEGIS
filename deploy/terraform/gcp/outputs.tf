# AEGIS — zokastech.fr — Apache 2.0 / MIT

output "vpc_network_id" {
  description = "ID du VPC custom."
  value       = google_compute_network.main.id
}

output "vpc_connector_id" {
  description = "ID du Serverless VPC Access Connector."
  value       = google_vpc_access_connector.main.id
}

output "cloud_run_gateway_uri" {
  description = "URL du service gateway Cloud Run (v2)."
  value       = google_cloud_run_v2_service.gateway.uri
}

output "cloud_run_core_name" {
  description = "Nom du service core (ingress interne uniquement)."
  value       = google_cloud_run_v2_service.core.name
}

output "cloud_run_dashboard_name" {
  description = "Nom du service dashboard (ingress interne)."
  value       = google_cloud_run_v2_service.dashboard.name
}

output "sql_private_ip" {
  description = "IP privée Cloud SQL (si activé)."
  value       = try(google_sql_database_instance.main[0].private_ip_address, null)
}

output "sql_connection_name" {
  description = "Nom de connexion Cloud SQL (pour proxy / connecteur)."
  value       = try(google_sql_database_instance.main[0].connection_name, null)
}

output "redis_host" {
  description = "Hôte Memorystore (si activé)."
  value       = try(google_redis_instance.main[0].host, null)
}

output "run_service_account_email" {
  description = "Compte de service utilisé par les révisions Cloud Run."
  value       = google_service_account.run.email
}
