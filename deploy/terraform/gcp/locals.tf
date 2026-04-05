# AEGIS — zokastech.fr — Apache 2.0 / MIT

locals {
  name = "aegis-${var.environment}"

  labels = merge(
    {
      environment = var.environment
      brand       = "aegis"
      org         = "zokastech"
    },
    var.default_labels
  )
}
