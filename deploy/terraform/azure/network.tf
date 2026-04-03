# AEGIS — zokastech.fr — Apache 2.0 / MIT

resource "azurerm_virtual_network" "main" {
  name                = "${local.name}-vnet"
  location            = var.location
  resource_group_name = var.resource_group_name
  address_space       = var.vnet_address_space
  tags                = local.tags

  depends_on = [azurerm_resource_group.main]
}

resource "azurerm_subnet" "containerapps" {
  name                 = "${local.name}-cae"
  resource_group_name  = var.resource_group_name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = [var.containerapps_subnet_cidr]

  delegation {
    name = "cae"
    service_delegation {
      name = "Microsoft.App/environments"
      actions = [
        "Microsoft.Network/virtualNetworks/subnets/join/action"
      ]
    }
  }

  depends_on = [azurerm_virtual_network.main]
}

resource "azurerm_subnet" "database" {
  count = var.enable_postgresql ? 1 : 0

  name                 = "${local.name}-db"
  resource_group_name  = var.resource_group_name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = [var.database_subnet_cidr]

  delegation {
    name = "pgsql"
    service_delegation {
      name = "Microsoft.DBforPostgreSQL/flexibleServers"
      actions = [
        "Microsoft.Network/virtualNetworks/subnets/join/action"
      ]
    }
  }

  depends_on = [azurerm_virtual_network.main]
}

resource "azurerm_network_security_group" "containerapps" {
  name                = "${local.name}-cae-nsg"
  location            = var.location
  resource_group_name = var.resource_group_name
  tags                = local.tags

  dynamic "security_rule" {
    for_each = var.enable_postgresql ? [1] : []
    content {
      name                         = "AllowPostgresOutbound"
      priority                     = 120
      direction                    = "Outbound"
      access                       = "Allow"
      protocol                     = "Tcp"
      source_port_range            = "*"
      destination_port_range       = "5432"
      source_address_prefix        = var.containerapps_subnet_cidr
      destination_address_prefixes = [var.database_subnet_cidr]
    }
  }

  security_rule {
    name                       = "AllowHttpsOutbound"
    priority                   = 200
    direction                  = "Outbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "443"
    source_address_prefix      = var.containerapps_subnet_cidr
    destination_address_prefix = "Internet"
  }

  depends_on = [azurerm_resource_group.main]
}

resource "azurerm_subnet_network_security_group_association" "containerapps" {
  subnet_id                 = azurerm_subnet.containerapps.id
  network_security_group_id = azurerm_network_security_group.containerapps.id
}
