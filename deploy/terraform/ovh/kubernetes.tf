# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Cluster managé OVHcloud — privilégier les régions UE (GRA, RBX, SBG, etc.) pour la souveraineté des données.

resource "ovh_cloud_project_kube" "aegis" {
  service_name = var.ovh_project_id
  name         = local.name
  region       = var.kube_region
  version      = var.kubernetes_version
}

resource "ovh_cloud_project_kube_nodepool" "workers" {
  service_name  = var.ovh_project_id
  kube_id       = ovh_cloud_project_kube.aegis.id
  name          = var.nodepool_name
  flavor_name   = var.node_flavor
  desired_nodes = var.node_desired
}
