# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Mot de passe maître géré par AWS (Secrets Manager) — référencer le secret dans la tâche ECS.

resource "aws_db_instance" "main" {
  count = var.rds_enabled ? 1 : 0

  identifier     = "${local.name}-pg"
  engine         = "postgres"
  engine_version = var.rds_engine_version
  instance_class = var.rds_instance_class

  allocated_storage     = var.rds_allocated_storage
  max_allocated_storage = var.rds_allocated_storage * 2
  storage_encrypted     = true

  db_name  = var.rds_database_name
  username = var.rds_username

  manage_master_user_password = true

  db_subnet_group_name   = module.vpc.database_subnet_group_name
  vpc_security_group_ids = [aws_security_group.rds[0].id]

  multi_az                = var.rds_multi_az
  backup_retention_period = var.rds_backup_retention_period

  skip_final_snapshot = var.environment != "prod"
  deletion_protection = var.environment == "prod"

  tags = { Name = "${local.name}-rds" }
}
