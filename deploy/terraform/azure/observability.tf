# AEGIS — zokastech.fr — Apache 2.0 / MIT

resource "azurerm_log_analytics_workspace" "main" {
  name                = "${local.name}-law"
  location            = var.location
  resource_group_name = var.resource_group_name
  sku                 = "PerGB2018"
  retention_in_days   = 30
  tags                = local.tags

  depends_on = [azurerm_resource_group.main]
}
