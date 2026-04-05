# AEGIS — zokastech.fr — Apache 2.0 / MIT
# ECS Fargate — gateway derrière ALB ; core et dashboard en réseau privé.

data "aws_iam_policy_document" "ecs_task_assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["ecs-tasks.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "ecs_execution" {
  name               = "${local.name}-ecs-exec"
  assume_role_policy = data.aws_iam_policy_document.ecs_task_assume.json
}

resource "aws_iam_role_policy_attachment" "ecs_execution" {
  role       = aws_iam_role.ecs_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

resource "aws_iam_role_policy" "execution_secrets" {
  count = var.rds_enabled ? 1 : 0

  name = "rds-master-secret"
  role = aws_iam_role.ecs_execution.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect   = "Allow"
      Action   = ["secretsmanager:GetSecretValue"]
      Resource = [aws_db_instance.main[0].master_user_secret[0].secret_arn]
    }]
  })
}

resource "aws_iam_role" "ecs_task" {
  name               = "${local.name}-ecs-task"
  assume_role_policy = data.aws_iam_policy_document.ecs_task_assume.json
}

resource "aws_ecs_cluster" "main" {
  name = local.name

  setting {
    name  = "containerInsights"
    value = "enabled"
  }

  tags = { Name = "${local.name}-ecs" }
}

locals {
  gateway_environment = concat(
    [
      { name = "AEGIS_POLICY", value = "/policies/gdpr-strict.yaml" }
    ],
    var.redis_enabled ? [{ name = "REDIS_ADDR", value = "${aws_elasticache_cluster.redis[0].cache_nodes[0].address}:6379" }] : [],
    var.rds_enabled ? [
      { name = "AEGIS_DB_HOST", value = aws_db_instance.main[0].address },
      { name = "AEGIS_DB_PORT", value = tostring(aws_db_instance.main[0].port) },
      { name = "AEGIS_DB_NAME", value = var.rds_database_name },
      { name = "AEGIS_DB_USER", value = var.rds_username }
    ] : []
  )

  gateway_secrets = var.rds_enabled ? [
    {
      name      = "AEGIS_DB_PASSWORD"
      valueFrom = "${aws_db_instance.main[0].master_user_secret[0].secret_arn}:password::"
    }
  ] : []
}

resource "aws_ecs_task_definition" "gateway" {
  family                   = "${local.name}-gateway"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = var.fargate_cpu
  memory                   = var.fargate_memory
  execution_role_arn       = aws_iam_role.ecs_execution.arn
  task_role_arn            = aws_iam_role.ecs_task.arn

  container_definitions = jsonencode([{
    name         = "gateway"
    image        = var.ecs_gateway_image
    essential    = true
    portMappings = [{ containerPort = var.gateway_container_port, protocol = "tcp" }]
    environment  = local.gateway_environment
    secrets      = local.gateway_secrets
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = aws_cloudwatch_log_group.gateway.name
        "awslogs-region"        = var.aws_region
        "awslogs-stream-prefix" = "gateway"
      }
    }
  }])

  tags = { Name = "${local.name}-td-gateway" }
}

resource "aws_ecs_task_definition" "core" {
  family                   = "${local.name}-core"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = var.fargate_cpu
  memory                   = var.fargate_memory
  execution_role_arn       = aws_iam_role.ecs_execution.arn
  task_role_arn            = aws_iam_role.ecs_task.arn

  container_definitions = jsonencode([{
    name      = "core"
    image     = var.ecs_core_image
    essential = true
    command   = ["sleep", "infinity"]
    environment = concat(
      var.redis_enabled ? [{ name = "REDIS_ADDR", value = "${aws_elasticache_cluster.redis[0].cache_nodes[0].address}:6379" }] : [],
      var.rds_enabled ? [
        { name = "AEGIS_DB_HOST", value = aws_db_instance.main[0].address },
        { name = "AEGIS_DB_PORT", value = tostring(aws_db_instance.main[0].port) },
        { name = "AEGIS_DB_NAME", value = var.rds_database_name },
        { name = "AEGIS_DB_USER", value = var.rds_username }
      ] : []
    )
    secrets = local.gateway_secrets
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = aws_cloudwatch_log_group.core.name
        "awslogs-region"        = var.aws_region
        "awslogs-stream-prefix" = "core"
      }
    }
  }])

  tags = { Name = "${local.name}-td-core" }
}

resource "aws_ecs_task_definition" "dashboard" {
  count = var.enable_dashboard_service ? 1 : 0

  family                   = "${local.name}-dashboard"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = min(var.fargate_cpu, 512)
  memory                   = min(var.fargate_memory, 1024)
  execution_role_arn       = aws_iam_role.ecs_execution.arn
  task_role_arn            = aws_iam_role.ecs_task.arn

  container_definitions = jsonencode([{
    name      = "dashboard"
    image     = var.ecs_dashboard_image
    essential = true
    portMappings = [{
      containerPort = 8080
      protocol      = "tcp"
    }]
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = aws_cloudwatch_log_group.dashboard[0].name
        "awslogs-region"        = var.aws_region
        "awslogs-stream-prefix" = "dashboard"
      }
    }
  }])

  tags = { Name = "${local.name}-td-dashboard" }
}

resource "aws_ecs_service" "gateway" {
  name            = "${local.name}-gateway"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.gateway.arn
  desired_count   = var.gateway_desired_count
  launch_type     = "FARGATE"

  network_configuration {
    subnets          = module.vpc.private_subnets
    security_groups  = [aws_security_group.ecs_tasks.id]
    assign_public_ip = false
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.gateway.arn
    container_name   = "gateway"
    container_port   = var.gateway_container_port
  }

  health_check_grace_period_seconds = 90

  # Listener HTTP toujours présent ; le HTTPS dépend du certificat ACM (apply éventuellement en deux temps).
  depends_on = [aws_lb_listener.http]
}

resource "aws_ecs_service" "core" {
  name            = "${local.name}-core"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.core.arn
  desired_count   = var.core_desired_count
  launch_type     = "FARGATE"

  network_configuration {
    subnets          = module.vpc.private_subnets
    security_groups  = [aws_security_group.ecs_tasks.id]
    assign_public_ip = false
  }
}

resource "aws_ecs_service" "dashboard" {
  count = var.enable_dashboard_service ? 1 : 0

  name            = "${local.name}-dashboard"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.dashboard[0].arn
  desired_count   = var.dashboard_desired_count
  launch_type     = "FARGATE"

  network_configuration {
    subnets          = module.vpc.private_subnets
    security_groups  = [aws_security_group.ecs_tasks.id]
    assign_public_ip = false
  }
}
