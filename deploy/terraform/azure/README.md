# AEGIS — zokastech.fr — Apache 2.0 / MIT

Module Terraform **Azure** pour AEGIS : **VNet**, sous-réseaux délégués (**Container Apps**, **PostgreSQL Flexible**, **Redis Premium**), **NSG** sur le sous-réseau Container Apps, **Container Apps Environment** avec **Log Analytics**, applications **gateway** (ingress public), **core** et **dashboard** (ingress interne).

## Prérequis

- Abonnement Azure, rôle suffisant pour créer RG, réseau, bases, cache, Container Apps.
- **Redis** : ce module provisionne un cache **Standard** (endpoint **public**, port TLS **6380**). Restreignez l’accès via le **firewall Redis** dans le portail ou des ressources complémentaires (**Private Endpoint**, Redis **Enterprise**). La délégation de sous-réseau `Microsoft.Cache/redis` n’est pas utilisée ici (validation schéma `azurerm_subnet` selon versions provider).
- **PostgreSQL** utilise un sous-réseau délégué `Microsoft.DBforPostgreSQL/flexibleServers` et une **zone DNS privée** `privatelink.postgres.database.azure.com`.

## Utilisation

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

Si le groupe de ressources existe déjà : `create_resource_group=false` et le même `resource_group_name`.

## Coûts et production

- Le SKU Redis **Premium** et les journaux Log Analytics ont un coût récurrent ; ajustez `postgresql_sku`, `redis_capacity` et la rétention des logs.
- Le mot de passe administrateur PostgreSQL est généré par Terraform (state sensible) : migrez vers **Key Vault** pour la rotation et l’audit.

## Licence

Apache-2.0 et MIT — **AEGIS** par [zokastech.fr](https://zokastech.fr).
