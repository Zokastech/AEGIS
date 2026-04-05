# AEGIS — zokastech.fr — Apache 2.0 / MIT

resource "google_service_account" "run" {
  account_id   = replace(substr("${local.name}-run", 0, 25), "_", "-")
  display_name = "AEGIS Cloud Run"
}

resource "google_project_iam_member" "run_sql" {
  count   = var.enable_cloud_sql ? 1 : 0
  project = var.project_id
  role    = "roles/cloudsql.client"
  member  = "serviceAccount:${google_service_account.run.email}"
}

resource "google_secret_manager_secret_iam_member" "run_sql_pw" {
  count     = var.enable_cloud_sql ? 1 : 0
  project   = var.project_id
  secret_id = google_secret_manager_secret.sql_password[0].secret_id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.run.email}"
}
