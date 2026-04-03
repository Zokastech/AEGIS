# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Règle east-west : restreignez les ports (5432, 6379 uniquement) selon votre durcissement.

resource "google_compute_firewall" "allow_internal" {
  name    = replace("${local.name}-int", "_", "-")
  network = google_compute_network.main.name

  allow {
    protocol = "tcp"
    ports    = ["0-65535"]
  }

  allow {
    protocol = "udp"
    ports    = ["0-65535"]
  }

  source_ranges = [var.vpc_cidr, var.connector_cidr]
  description   = "Trafic interne VPC + plage du connecteur Serverless VPC Access."
}
