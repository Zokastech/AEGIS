# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Cloud providers — Terraform (AWS, GCP, Azure, OVHcloud)

This page summarizes the **root Terraform modules** under [`deploy/terraform/`](https://github.com/zokastech/aegis/tree/main/deploy/terraform) for running **AEGIS** on major clouds. Each folder is self-contained (variables, outputs, README). **Review and harden** before production.

Overview table: [`deploy/terraform/README.md`](https://github.com/zokastech/aegis/blob/main/deploy/terraform/README.md).

---

## Amazon Web Services (AWS)

**Path:** `deploy/terraform/aws/`

**What it provisions:** VPC (public / private / database tiers), **ECS Fargate** (gateway, core, dashboard), **ALB** with optional **WAFv2**, **ACM**, **RDS PostgreSQL**, **ElastiCache Redis**, **CloudWatch Logs**.

**Flow:** Gateway is registered on the ALB (health check `/health/ready`). Core and dashboard run in private subnets without an ALB target by default (extend if you need external access).

**Quick start:**

```bash
cd deploy/terraform/aws
terraform init
terraform plan \
  -var="project_name=aegis" \
  -var="environment=prod" \
  -var="aws_region=eu-west-3" \
  -var="acm_domain_name=api.example.com" \
  -var="create_route53_records=true" \
  -var="route53_zone_id=Zxxxxxxxx"
terraform apply
```

**Notes:** RDS master password via **Secrets Manager**; ECS task role can inject `AEGIS_DB_PASSWORD`. TLS: DNS validation with Route53 when configured.

**Full guide:** [AWS module README](https://github.com/zokastech/aegis/blob/main/deploy/terraform/aws/README.md).

---

## Google Cloud Platform (GCP)

**Path:** `deploy/terraform/gcp/`

**What it provisions:** Custom **VPC**, **Serverless VPC Access**, **Cloud Run** (v2) for gateway / core / dashboard, **Cloud SQL** PostgreSQL (private IP + Service Networking), **Memorystore** Redis, baseline **firewalls**, **Secret Manager** for DB password.

**APIs:** Enable at least `run.googleapis.com`, `vpcaccess.googleapis.com`, `servicenetworking.googleapis.com`, `sqladmin.googleapis.com`, `redis.googleapis.com`, `secretmanager.googleapis.com`, `compute.googleapis.com` (see README for `gcloud` one-liner).

**Quick start:**

```bash
cd deploy/terraform/gcp
export GOOGLE_PROJECT=your-project
export GOOGLE_REGION=europe-west9
terraform init
terraform apply \
  -var="project_id=$GOOGLE_PROJECT" \
  -var="environment=prod"
```

**Notes:** Cloud Run uses `PRIVATE_RANGES_ONLY` through the connector to reach SQL/Redis. Default **`allow_public_cloud_run`** is `false` — use IAM `roles/run.invoker` or set `true` only for tests. Gateway is public URL; core/dashboard internal-only by default.

**Full guide:** [GCP module README](https://github.com/zokastech/aegis/blob/main/deploy/terraform/gcp/README.md).

---

## Microsoft Azure

**Path:** `deploy/terraform/azure/`

**What it provisions:** **VNet**, delegated subnets (**Container Apps**, **PostgreSQL Flexible**, **Redis**), **NSG** on the Container Apps subnet, **Container Apps Environment** with **Log Analytics**, apps: **gateway** (public ingress), **core** and **dashboard** (internal ingress).

**Quick start:**

```bash
cd deploy/terraform/azure
az login
terraform init
terraform apply \
  -var="resource_group_name=aegis-rg" \
  -var="environment=prod" \
  -var="create_resource_group=true" \
  -var="location=francecentral"
```

**Notes:** Redis is **Standard** with a **public** endpoint (TLS port **6380**) — restrict via Redis firewall or add **Private Endpoint** / Enterprise tier for stricter isolation. PostgreSQL uses private DNS zone `privatelink.postgres.database.azure.com`. Consider **Key Vault** for password rotation instead of Terraform-only secrets in state.

**Full guide:** [Azure module README](https://github.com/zokastech/aegis/blob/main/deploy/terraform/azure/README.md).

---

## OVHcloud (EU / sovereign-friendly)

**Path:** `deploy/terraform/ovh/`

**What it provisions:** **Managed Kubernetes** (cluster + worker node pool), optional **Object Storage** (S3-compatible) for **ONNX models** and heavy artifacts.

**Compliance:** Pick `kube_region` in the EU footprint (e.g. **GRA**, **RBX**, **SBG**, **DE**, **WAW** per OVH catalog). This module does not replace **SecNumCloud** / **HDS** / ANSSI-level qualification — harden with NetworkPolicies, encryption, and IAM via **`deploy/helm/aegis`**.

**Auth:** Set `OVH_APPLICATION_KEY`, `OVH_APPLICATION_SECRET`, `OVH_CONSUMER_KEY` (provider endpoint **ovh-eu**).

**Object Storage:** Create OpenStack user with Object Storage role, S3 keys, then set `object_storage_s3_endpoint`, `object_storage_s3_region`, keys, and `create_models_bucket` / `models_bucket_name`. The **hashicorp/aws** provider is used **only** as an S3 client against OVH’s endpoint (no AWS public resources).

**Quick start:**

```bash
cd deploy/terraform/ovh
export OVH_APPLICATION_KEY=...
export OVH_APPLICATION_SECRET=...
export OVH_CONSUMER_KEY=...
terraform init
terraform apply \
  -var="ovh_project_id=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" \
  -var="environment=prod"
```

Then use kubeconfig (output `kubeconfig_yaml` or OVH console) and:

```bash
helm install aegis ./deploy/helm/aegis --namespace aegis --create-namespace
```

**Full guide:** [OVH module README](https://github.com/zokastech/aegis/blob/main/deploy/terraform/ovh/README.md).

---

## Related

- [Deployment](deployment.md) — Docker Compose, Helm, secrets, observability
- [Security](security/index.md) — hardening and compliance orientation
