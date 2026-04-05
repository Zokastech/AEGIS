# AEGIS — zokastech.fr — Apache 2.0 / MIT

resource "google_redis_instance" "main" {
  count              = var.enable_memorystore ? 1 : 0
  name               = replace("${local.name}-redis", "_", "-")
  tier               = var.redis_tier
  memory_size_gb     = var.redis_memory_gb
  region             = var.region
  authorized_network = google_compute_network.main.id
  connect_mode       = "PRIVATE_SERVICE_ACCESS"
  reserved_ip_range  = google_compute_global_address.redis_peering[0].name

  depends_on = [google_service_networking_connection.private]
}
