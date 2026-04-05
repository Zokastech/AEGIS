# AEGIS — zokastech.fr — Apache 2.0 / MIT

output "kube_cluster_id" {
  description = "Identifiant interne du cluster OVH Kubernetes."
  value       = ovh_cloud_project_kube.aegis.id
}

output "kubeconfig_yaml" {
  description = "Kubeconfig complet (sensible) — à stocker hors dépôt (vault, SOPS)."
  value       = ovh_cloud_project_kube.aegis.kubeconfig
  sensitive   = true
}

output "nodepool_id" {
  description = "ID du node pool worker."
  value       = ovh_cloud_project_kube_nodepool.workers.id
}

output "models_bucket_id" {
  description = "Nom du bucket modèles (Object Storage S3), si créé."
  value       = try(aws_s3_bucket.models[0].id, null)
}

output "models_bucket_arn" {
  description = "ARN logique du bucket (provider S3 OVH)."
  value       = try(aws_s3_bucket.models[0].arn, null)
}
