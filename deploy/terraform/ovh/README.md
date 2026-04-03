# AEGIS — zokastech.fr — Apache 2.0 / MIT

Module Terraform **OVHcloud** pour déployer AEGIS sur une infrastructure **souveraine UE** : **Managed Kubernetes** (création du cluster et d’un **node pool** workers), option **Object Storage** compatible **S3** pour héberger les **modèles ONNX** et autres artefacts lourds (évite de les embarquer dans l’image conteneur).

## Conformité et résidence des données (UE)

- **Localisation** : choisissez `kube_region` dans le périmètre européen OVHcloud (ex. **GRA** Gravelines, **RBX** Roubaix, **SBG** Strasbourg, **DE** Francfort, **WAW** Varsovie selon catalogue). Vérifiez le **DPA** OVHcloud et votre **registre des traitements** (RGPD) : localisation des backups, logs plan de contrôle, support technique.
- **Schéma recommandé** : données personnelles traitées par **aegis-gateway** / **aegis-core** avec politiques de minimisation (voir docs AEGIS RGPD) ; modèles et journaux applicatifs dans **Object Storage** et bases **PostgreSQL** / **Redis** hébergés dans la même juridiction que vos exigences contractuelles.
- **Ce module** ne remplace pas une **ANSSI** / **HDS** / **SecNumCloud** qualification : adaptez durcissement (NetworkPolicy, chiffrement, IAM fines) via le chart Helm `deploy/helm/aegis` et vos politiques internes.

## Authentification OVH

Créez un token API (droits **GET/POST** sur `/cloud/**` selon besoin) ou utilisez les variables d’environnement documentées par le provider :

- `OVH_APPLICATION_KEY`
- `OVH_APPLICATION_SECRET`
- `OVH_CONSUMER_KEY`

Endpoint provider : **ovh-eu** (défaut dans `providers.tf`).

## Object Storage (S3)

1. Créez un **utilisateur OpenStack** avec rôle **Object Storage operator** (console OVHcloud).
2. Générez des **clés S3** pour cet utilisateur.
3. Renseignez `object_storage_s3_endpoint` (ex. `https://s3.gra.io.cloud.ovh.net`), `object_storage_s3_region` (ex. `gra`), `object_storage_access_key`, `object_storage_secret_key`.
4. `create_models_bucket = true` et optionnellement `models_bucket_name` (sinon nom dérivé de `cluster_name` + `environment`).

Le provider **hashicorp/aws** est utilisé **uniquement** comme client S3 vers l’endpoint OVH (`skip_*` activés) ; il ne provisionne rien chez AWS public.

## Utilisation

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

Récupérez le kubeconfig via l’output sensible `kubeconfig_yaml` ou la console OVHcloud, puis :

```bash
helm install aegis ./deploy/helm/aegis --namespace aegis --create-namespace
```

## Licence

Apache-2.0 et MIT — **AEGIS** par [zokastech.fr](https://zokastech.fr).
