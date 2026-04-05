# AEGIS — zokastech.fr — Apache 2.0 / MIT

locals {
  name = "aegis-${var.environment}"

  tags = merge(
    {
      environment = var.environment
      brand       = "AEGIS"
      org         = "zokastech.fr"
    },
    var.tags
  )
}
