# AEGIS — zokastech.fr — Apache 2.0 / MIT

Chart Helm 3 pour déployer la stack **AEGIS** (zokastech.fr) sur Kubernetes : **aegis-core**, **aegis-gateway**, **aegis-dashboard**, Redis optionnel (StatefulSet), Ingress (TLS / cert-manager), NetworkPolicy, RBAC, PDB, HPA (CPU et/ou métriques custom Prometheus).

## Prérequis

- **Kubernetes** ≥ 1.25 (PodSecurityPolicy retiré ; le chart applique des **SecurityContext** restrictifs et des labels indicatifs PSS, pas de PSP).
- **Helm** 3.
- **HPA sur métrique Prometheus** : cluster avec **prometheus-adapter** (ou équivalent) exposant les noms configurés dans `core.autoscaling.prometheus.metricName` / `gateway.autoscaling.prometheus.metricName` via l’API `custom.metrics.k8s.io`.
- **TLS automatique** : **cert-manager** + un `ClusterIssuer` (ex. Let’s Encrypt) si vous utilisez les annotations suggérées dans `values.yaml`.

## Installation

Depuis le dépôt (chart en chemin local) :

```bash
helm install aegis ./deploy/helm/aegis --namespace aegis --create-namespace
```

Une fois le dépôt de charts publié :

```bash
helm repo add aegis https://charts.zokastech.fr
helm install aegis aegis/aegis --namespace aegis --create-namespace
```

Vérification locale du rendu (utile si votre contexte `kubectl` pointe vers une vieille version API) :

```bash
helm template aegis ./deploy/helm/aegis --kube-version 1.28.0
helm lint ./deploy/helm/aegis
```

## Composants principaux

| Composant | Ressource | Rôle |
|-----------|-----------|------|
| **core** | `Deployment` (+ PVC optionnel, ConfigMap moteur si `core.configMap.create`) | Worker / moteur (pas d’exposition HTTP par défaut ; commande type `sleep infinity` à remplacer selon l’image). |
| **gateway** | `Deployment`, `Service` (HTTP + gRPC), `Secret` env optionnel, ConfigMap politiques | API HTTP/HTTPS, métriques, probes `/health/live` et `/health/ready`. |
| **dashboard** | `Deployment`, `Service` | UI statique (nginx) ; `containerPort` vs `service.port` configurables. |
| **redis** | `StatefulSet` (si `redis.enabled`) | Cache embarqué — préférer un Redis managé en production. |

Ressources optionnelles selon les valeurs : **Ingress**, **NetworkPolicy**, **ServiceMonitor** (Prometheus Operator), **HPA**, **PDB**, **Role/RoleBinding**.

## Ingress et Let’s Encrypt (cert-manager)

Dans `values.yaml`, activez `ingress.enabled: true` et renseignez `ingress.hosts` / `ingress.tls`. Exemple d’annotations (à adapter à votre issuer) :

```yaml
ingress:
  enabled: true
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  tls:
    - secretName: aegis-tls
      hosts:
        - aegis.example.com
```

## Autoscaling (HPA)

- **CPU** : `core.autoscaling` / `gateway.autoscaling` avec `cpuMetric: true` et `targetCPUUtilizationPercentage`.
- **Prometheus** : `prometheus.enabled: true`, `metricName` et `averageValue` (chaîne Quantité Kubernetes, ex. `"100"` ou `"500m"`). Le chart échoue au rendu si l’autoscaling est activé sans aucune métrique (ni CPU ni Prometheus).

## Images

Par défaut : `ghcr.io/zokastech/aegis-core`, `aegis-gateway`, `aegis-dashboard`. Surcharge via `--set` ou un fichier `-f values-prod.yaml`.

## Licence

Apache-2.0 et MIT — **AEGIS** par [zokastech.fr](https://zokastech.fr).
