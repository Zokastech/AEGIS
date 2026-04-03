# AEGIS — zokastech.fr — Apache 2.0 / MIT

resource "google_compute_network" "main" {
  name                    = var.network_name
  auto_create_subnetworks = false
  routing_mode            = "REGIONAL"
}

resource "google_compute_subnetwork" "main" {
  name          = "${var.network_name}-${var.region}"
  ip_cidr_range = cidrsubnet(var.vpc_cidr, 4, 0)
  region        = var.region
  network       = google_compute_network.main.id
}

resource "google_compute_global_address" "sql_peering" {
  count = var.enable_cloud_sql ? 1 : 0

  name          = "${local.name}-sql-peer"
  purpose       = "VPC_PEERING"
  address_type  = "INTERNAL"
  prefix_length = var.sql_peering_prefix_length
  network       = google_compute_network.main.id
}

resource "google_compute_global_address" "redis_peering" {
  count = var.enable_memorystore ? 1 : 0

  name          = "${local.name}-redis-peer"
  purpose       = "VPC_PEERING"
  address_type  = "INTERNAL"
  prefix_length = 24
  network       = google_compute_network.main.id
}

locals {
  peering_range_names = concat(
    [for a in google_compute_global_address.sql_peering : a.name],
    [for a in google_compute_global_address.redis_peering : a.name],
  )
}

resource "google_service_networking_connection" "private" {
  count = length(local.peering_range_names) > 0 ? 1 : 0

  network                 = google_compute_network.main.id
  service                 = "servicenetworking.googleapis.com"
  reserved_peering_ranges = local.peering_range_names
}

resource "google_vpc_access_connector" "main" {
  name          = replace("${local.name}-conn", "_", "-")
  region        = var.region
  network       = google_compute_network.main.name
  ip_cidr_range = var.connector_cidr
}
