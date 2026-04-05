# AEGIS — zokastech.fr — Apache 2.0 / MIT

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = merge(
      var.default_tags,
      {
        Project      = var.project_name
        Environment  = var.environment
        ManagedBy    = "terraform"
        Brand        = "AEGIS"
        Organization = "zokastech.fr"
      }
    )
  }
}
