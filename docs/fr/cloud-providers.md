# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Fournisseurs cloud — Terraform (AWS, GCP, Azure, OVHcloud)

Cette page résume les **modules Terraform racine** sous [`deploy/terraform/`](https://github.com/zokastech/aegis/tree/main/deploy/terraform) pour exécuter **AEGIS** sur les principaux clouds. Chaque dossier est autonome (variables, sorties, README). **À relire et durcir** avant la production.

Tableau d’ensemble : [`deploy/terraform/README.md`](https://github.com/zokastech/aegis/blob/main/deploy/terraform/README.md).

---

## Amazon Web Services (AWS)

**Chemin :** `deploy/terraform/aws/`

**Ressources :** VPC (public / privé / données), **ECS Fargate** (gateway, core, dashboard), **ALB** avec **WAFv2** optionnel, **ACM**, **RDS PostgreSQL**, **ElastiCache Redis**, **CloudWatch Logs**.

**Flux :** La gateway est enregistrée sur l’ALB (health check `/health/ready`). Le core et le dashboard tournent en sous-réseaux privés sans cible ALB par défaut (à étendre si besoin d’accès externe).

**Démarrage rapide :**

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

**À noter :** mot de passe RDS via **Secrets Manager** ; le rôle d’exécution ECS peut injecter `AEGIS_DB_PASSWORD`. TLS : validation DNS avec Route53 si configuré.

**Guide complet :** [README module AWS](https://github.com/zokastech/aegis/blob/main/deploy/terraform/aws/README.md).

---

## Google Cloud Platform (GCP)

**Chemin :** `deploy/terraform/gcp/`

**Ressources :** **VPC** dédié, **Serverless VPC Access**, **Cloud Run** (v2) pour gateway / core / dashboard, **Cloud SQL** PostgreSQL (IP privée + Service Networking), **Memorystore** Redis, **firewalls** de base, **Secret Manager** pour le mot de passe base.

**APIs :** activer au minimum `run.googleapis.com`, `vpcaccess.googleapis.com`, `servicenetworking.googleapis.com`, `sqladmin.googleapis.com`, `redis.googleapis.com`, `secretmanager.googleapis.com`, `compute.googleapis.com` (voir le README pour la commande `gcloud`).

**Démarrage rapide :**

```bash
cd deploy/terraform/gcp
export GOOGLE_PROJECT=your-project
export GOOGLE_REGION=europe-west9
terraform init
terraform apply \
  -var="project_id=$GOOGLE_PROJECT" \
  -var="environment=prod"
```

**À noter :** Cloud Run utilise `PRIVATE_RANGES_ONLY` via le connecteur pour joindre SQL/Redis. Par défaut **`allow_public_cloud_run`** est `false` — utiliser IAM `roles/run.invoker` ou passer à `true` uniquement pour des tests. Gateway en URL publique ; core/dashboard en interne par défaut.

**Guide complet :** [README module GCP](https://github.com/zokastech/aegis/blob/main/deploy/terraform/gcp/README.md).

---

## Microsoft Azure

**Chemin :** `deploy/terraform/azure/`

**Ressources :** **VNet**, sous-réseaux délégués (**Container Apps**, **PostgreSQL Flexible**, **Redis**), **NSG** sur le sous-réseau Container Apps, **Container Apps Environment** avec **Log Analytics**, applications **gateway** (ingress public), **core** et **dashboard** (ingress interne).

**Démarrage rapide :**

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

**À noter :** Redis en **Standard** avec endpoint **public** (TLS port **6380**) — restreindre via pare-feu Redis ou **Private Endpoint** / offre Enterprise pour un isolement plus strict. PostgreSQL avec zone DNS privée `privatelink.postgres.database.azure.com`. Envisager **Key Vault** pour la rotation des secrets plutôt que des secrets uniquement dans le state Terraform.

**Guide complet :** [README module Azure](https://github.com/zokastech/aegis/blob/main/deploy/terraform/azure/README.md).

---

## OVHcloud (UE / souveraineté)

**Chemin :** `deploy/terraform/ovh/`

**Ressources :** **Managed Kubernetes** (cluster + pool de nœuds workers), **Object Storage** compatible **S3** optionnel pour les **modèles ONNX** et artefacts lourds.

**Conformité :** choisir `kube_region` dans le périmètre européen OVHcloud (ex. **GRA**, **RBX**, **SBG**, **DE**, **WAW** selon catalogue). Ce module ne remplace pas une qualification **SecNumCloud** / **HDS** / ANSSI — durcir avec NetworkPolicy, chiffrement, IAM via **`deploy/helm/aegis`**.

**Auth :** `OVH_APPLICATION_KEY`, `OVH_APPLICATION_SECRET`, `OVH_CONSUMER_KEY` (endpoint provider **ovh-eu**).

**Object Storage :** utilisateur OpenStack avec rôle Object Storage, clés S3, puis `object_storage_s3_endpoint`, `object_storage_s3_region`, clés, et `create_models_bucket` / `models_bucket_name`. Le provider **hashicorp/aws** sert **uniquement** de client S3 vers l’endpoint OVH (aucune ressource AWS public).

**Démarrage rapide :**

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

Puis récupérer le kubeconfig (sortie sensible `kubeconfig_yaml` ou console OVH) et :

```bash
helm install aegis ./deploy/helm/aegis --namespace aegis --create-namespace
```

**Guide complet :** [README module OVH](https://github.com/zokastech/aegis/blob/main/deploy/terraform/ovh/README.md).

---

## Voir aussi

- [Déploiement](deployment.md) — Docker Compose, Helm, secrets, observabilité
- [Sécurité](security/index.md) — durcissement et orientation conformité
