# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Cache Standard avec endpoint public (TLS). Pour une injection VNet, utilisez Redis Enterprise
# ou un Private Endpoint (azurerm_private_endpoint) — non inclus ici car la délégation
# Microsoft.Cache/redis n’est pas exposée de façon fiable dans tous les schémas azurerm_subnet.

resource "random_id" "redis_name" {
  count       = var.enable_redis ? 1 : 0
  byte_length = 3
}

resource "azurerm_redis_cache" "main" {
  count = var.enable_redis ? 1 : 0

  name                = "${replace(local.name, "-", "")}r${random_id.redis_name[0].hex}"
  location            = var.location
  resource_group_name = var.resource_group_name
  capacity            = var.redis_capacity
  family              = var.redis_family
  sku_name            = var.redis_sku_name

  minimum_tls_version  = "1.2"
  non_ssl_port_enabled = false

  tags = local.tags
}
