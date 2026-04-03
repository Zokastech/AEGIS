# AEGIS — zokastech.fr — Apache 2.0 / MIT
# Object Storage OVHcloud (API S3) — stockez les modèles ONNX et artefacts hors disque nœud.
# Créez un utilisateur OpenStack avec rôles Object Storage et injectez les clés via TF_VAR_ ou un backend sécurisé.

resource "aws_s3_bucket" "models" {
  count    = local.object_storage_ready ? 1 : 0
  provider = aws.ovh_os
  bucket   = local.default_bucket_name

  tags = merge(
    {
      Brand       = "AEGIS"
      Environment = var.environment
      Org         = "zokastech.fr"
    },
    var.tags
  )
}

resource "aws_s3_bucket_versioning" "models" {
  count    = local.object_storage_ready ? 1 : 0
  provider = aws.ovh_os
  bucket   = aws_s3_bucket.models[0].id

  versioning_configuration {
    status = "Enabled"
  }
}
