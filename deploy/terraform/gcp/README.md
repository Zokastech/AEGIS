# AEGIS — zokastech.fr — Apache 2.0 / MIT

Module Terraform **GCP** pour AEGIS : **VPC** custom, **Serverless VPC Access**, **Cloud Run** (v2) pour gateway / core / dashboard, **Cloud SQL** PostgreSQL (IP privée + Service Networking), **Memorystore** Redis (Private Service Access), règles **firewall** de base, **Secret Manager** pour le mot de passe base (injecté dans le gateway).

## APIs à activer

Activez au minimum : `run.googleapis.com`, `vpcaccess.googleapis.com`, `servicenetworking.googleapis.com`, `sqladmin.googleapis.com`, `redis.googleapis.com`, `secretmanager.googleapis.com`, `compute.googleapis.com`.

```bash
gcloud services enable run.googleapis.com vpcaccess.googleapis.com servicenetworking.googleapis.com \
  sqladmin.googleapis.com redis.googleapis.com secretmanager.googleapis.com compute.googleapis.com \
  --project=YOUR_PROJECT_ID
```

## Utilisation

```bash
cd deploy/terraform/gcp
export GOOGLE_PROJECT=your-project
export GOOGLE_REGION=europe-west9
terraform init
terraform apply \
  -var="project_id=$GOOGLE_PROJECT" \
  -var="environment=prod"
```

## Sécurité réseau

- Cloud Run utilise `PRIVATE_RANGES_ONLY` vers le connecteur pour joindre SQL/Redis en IP privée.
- Par défaut **`allow_public_cloud_run`** est `false` : ajoutez des bindings IAM `roles/run.invoker` pour vos identités ; passez à `true` uniquement pour des tests.
- Le **gateway** est en `INGRESS_TRAFFIC_ALL` (URL managée) ; **core** et **dashboard** sont en **internal only** (appels depuis le même projet / réseau / Load Balancer interne selon votre design).

## Conformité

Paramétrez `region` sur une région UE (ex. `europe-west9`) et documentez le traitement des données (RGPD) dans vos registres ; ce module ne configure pas DLP ni CMEK — à ajouter selon vos exigences.

## Licence

Apache-2.0 et MIT — **AEGIS** par [zokastech.fr](https://zokastech.fr).
