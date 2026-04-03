# AEGIS — zokastech.fr — Apache 2.0 / MIT

output "vpc_id" {
  description = "ID du VPC AEGIS."
  value       = module.vpc.vpc_id
}

output "public_subnet_ids" {
  description = "Sous-réseaux publics (ALB)."
  value       = module.vpc.public_subnets
}

output "private_subnet_ids" {
  description = "Sous-réseaux privés (ECS Fargate)."
  value       = module.vpc.private_subnets
}

output "database_subnet_ids" {
  description = "Sous-réseaux isolés données (RDS, ElastiCache)."
  value       = module.vpc.database_subnets
}

output "alb_dns_name" {
  description = "Nom DNS de l’ALB (CNAME cible avant cutover DNS)."
  value       = aws_lb.main.dns_name
}

output "alb_zone_id" {
  description = "Zone hébergée de l’ALB (alias Route 53)."
  value       = aws_lb.main.zone_id
}

output "ecs_cluster_name" {
  description = "Nom du cluster ECS."
  value       = aws_ecs_cluster.main.name
}

output "ecs_cluster_arn" {
  description = "ARN du cluster ECS."
  value       = aws_ecs_cluster.main.arn
}

output "acm_certificate_arn" {
  description = "ARN du certificat ACM (si acm_domain_name renseigné)."
  value       = try(aws_acm_certificate.main[0].arn, null)
}

output "rds_endpoint" {
  description = "Endpoint PostgreSQL (sans activer rds : null)."
  value       = try(aws_db_instance.main[0].endpoint, null)
}

output "rds_master_user_secret_arn" {
  description = "ARN du secret Secrets Manager du compte maître RDS (mot de passe géré par AWS)."
  value       = try(aws_db_instance.main[0].master_user_secret[0].secret_arn, null)
  sensitive   = true
}

output "redis_primary_endpoint" {
  description = "Adresse Redis ElastiCache (nœud primaire)."
  value       = try(aws_elasticache_cluster.redis[0].cache_nodes[0].address, null)
}

output "cloudwatch_log_groups" {
  description = "Groupes de journaux CloudWatch pour les services ECS."
  value = compact([
    aws_cloudwatch_log_group.gateway.name,
    aws_cloudwatch_log_group.core.name,
    var.enable_dashboard_service ? aws_cloudwatch_log_group.dashboard[0].name : null
  ])
}

output "waf_web_acl_arn" {
  description = "ARN du Web ACL WAFv2 (si waf_enabled)."
  value       = try(aws_wafv2_web_acl.main[0].arn, null)
}
