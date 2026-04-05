# AEGIS — zokastech.fr — Apache 2.0 / MIT

resource "aws_cloudwatch_log_group" "gateway" {
  name              = "/ecs/${local.name}/gateway"
  retention_in_days = var.cloudwatch_log_retention_days
}

resource "aws_cloudwatch_log_group" "core" {
  name              = "/ecs/${local.name}/core"
  retention_in_days = var.cloudwatch_log_retention_days
}

resource "aws_cloudwatch_log_group" "dashboard" {
  count = var.enable_dashboard_service ? 1 : 0

  name              = "/ecs/${local.name}/dashboard"
  retention_in_days = var.cloudwatch_log_retention_days
}
