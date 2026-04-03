# AEGIS — zokastech.fr — Apache 2.0 / MIT

resource "aws_lb" "main" {
  name               = substr(replace(local.name, "_", "-"), 0, 32)
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets            = module.vpc.public_subnets

  enable_deletion_protection = var.alb_deletion_protection

  tags = { Name = "${local.name}-alb" }
}

resource "aws_lb_target_group" "gateway" {
  name        = substr("${replace(local.name, "_", "-")}-gw", 0, 32)
  port        = var.gateway_container_port
  protocol    = "HTTP"
  vpc_id      = module.vpc.vpc_id
  target_type = "ip"

  health_check {
    enabled             = true
    healthy_threshold   = 2
    interval            = 30
    matcher             = "200"
    path                = "/health/ready"
    port                = "traffic-port"
    protocol            = "HTTP"
    timeout             = 5
    unhealthy_threshold = 3
  }

  tags = { Name = "${local.name}-tg-gateway" }
}

resource "aws_lb_listener" "http" {
  load_balancer_arn = aws_lb.main.arn
  port              = 80
  protocol          = "HTTP"

  default_action {
    type = var.acm_domain_name != "" ? "redirect" : "forward"

    dynamic "redirect" {
      for_each = var.acm_domain_name != "" ? [1] : []
      content {
        port        = "443"
        protocol    = "HTTPS"
        status_code = "HTTP_301"
      }
    }

    dynamic "forward" {
      for_each = var.acm_domain_name == "" ? [1] : []
      content {
        target_group {
          arn = aws_lb_target_group.gateway.arn
        }
      }
    }
  }
}

resource "aws_lb_listener" "https" {
  count = var.acm_domain_name != "" ? 1 : 0

  load_balancer_arn = aws_lb.main.arn
  port              = 443
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"
  certificate_arn   = var.create_route53_records && var.route53_zone_id != "" ? aws_acm_certificate_validation.main[0].certificate_arn : aws_acm_certificate.main[0].arn

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.gateway.arn
  }

  depends_on = [aws_acm_certificate.main]
}
