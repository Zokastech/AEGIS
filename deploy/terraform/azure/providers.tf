# AEGIS — zokastech.fr — Apache 2.0 / MIT

provider "azurerm" {
  features {
    resource_group {
      prevent_deletion_if_contains_resources = false
    }
  }
}
