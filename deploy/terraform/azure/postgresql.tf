# AEGIS — zokastech.fr — Apache 2.0 / MIT

resource "random_password" "postgres" {
  count   = var.enable_postgresql ? 1 : 0
  length  = 24
  special = true
}

resource "azurerm_private_dns_zone" "postgres" {
  count               = var.enable_postgresql ? 1 : 0
  name                = "privatelink.postgres.database.azure.com"
  resource_group_name = var.resource_group_name
  tags                = local.tags

  depends_on = [azurerm_resource_group.main]
}

resource "azurerm_private_dns_zone_virtual_network_link" "postgres" {
  count = var.enable_postgresql ? 1 : 0

  name                  = "${local.name}-pg-vnet"
  resource_group_name   = var.resource_group_name
  private_dns_zone_name = azurerm_private_dns_zone.postgres[0].name
  virtual_network_id    = azurerm_virtual_network.main.id
  tags                  = local.tags
}

resource "azurerm_postgresql_flexible_server" "main" {
  count = var.enable_postgresql ? 1 : 0

  name                          = "${local.name}-pg"
  resource_group_name           = var.resource_group_name
  location                      = var.location
  version                       = "16"
  administrator_login           = var.postgresql_admin_user
  administrator_password        = random_password.postgres[0].result
  sku_name                      = var.postgresql_sku
  storage_mb                    = var.postgresql_storage_mb
  delegated_subnet_id           = azurerm_subnet.database[0].id
  private_dns_zone_id           = azurerm_private_dns_zone.postgres[0].id
  public_network_access_enabled = false
  tags                          = local.tags

  depends_on = [azurerm_private_dns_zone_virtual_network_link.postgres[0]]
}

resource "azurerm_postgresql_flexible_server_database" "app" {
  count     = var.enable_postgresql ? 1 : 0
  name      = var.postgresql_database_name
  server_id = azurerm_postgresql_flexible_server.main[0].id
  charset   = "UTF8"
  collation = "en_US.utf8"
}
