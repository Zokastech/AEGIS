# AEGIS — zokastech.fr — Apache 2.0 / MIT

resource "aws_elasticache_subnet_group" "redis" {
  count = var.redis_enabled ? 1 : 0

  name       = "${local.name}-redis"
  subnet_ids = module.vpc.database_subnets
}

resource "aws_elasticache_cluster" "redis" {
  count = var.redis_enabled ? 1 : 0

  cluster_id           = "${local.name}-redis"
  engine               = "redis"
  node_type            = var.redis_node_type
  num_cache_nodes      = var.redis_num_cache_nodes
  parameter_group_name = "default.redis7"
  engine_version       = var.redis_engine_version
  port                 = 6379
  subnet_group_name    = aws_elasticache_subnet_group.redis[0].name
  security_group_ids   = [aws_security_group.redis[0].id]

  tags = { Name = "${local.name}-redis" }
}
