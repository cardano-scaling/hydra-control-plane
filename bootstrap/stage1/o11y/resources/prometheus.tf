resource "kubernetes_manifest" "prometheus" {
  manifest = {
    "apiVersion" = "monitoring.coreos.com/v1"
    "kind"       = "Prometheus"
    "metadata" = {
      "name"      = "prometheus"
      "namespace" = var.namespace
    }
    "spec" = {
      "alerting" = {
        "alertmanagers" = [
          {
            "apiVersion" = "v2"
            "name"       = "alertmanager"
            "namespace"  = var.namespace
            "port"       = "web"
          },
        ]
      }
      "shards"         = 2
      "enableAdminAPI" = false
      "externalLabels" = {
        "cluster" : var.cluster_name
      }
      "podMonitorNamespaceSelector" = {}
      "podMonitorSelector" = {
        "matchLabels" = {
          "app.kubernetes.io/component" = "o11y"
          "app.kubernetes.io/part-of"   = "hydradoom"
        }
      }
      "resources" = {
        "requests" = {
          "cpu"    = "1"
          "memory" = "13Gi"
        }
        "limits" = {
          "memory" = "13Gi"
        }
      }
      "retention"             = "30d"
      "ruleNamespaceSelector" = {}
      "ruleSelector"          = {}
      "scrapeInterval"        = "30s"
      "securityContext" = {
        "fsGroup" = 65534
      }
      "serviceAccountName"              = "prometheus"
      "serviceMonitorNamespaceSelector" = {}
      "serviceMonitorSelector" = {
        "matchLabels" = {
          "app.kubernetes.io/component" = "o11y"
          "app.kubernetes.io/part-of"   = "hydradoom"
        }
      }
      "storage" = {
        "volumeClaimTemplate" = {
          "spec" = {
            "storageClassName" = var.storage_class
            "resources" = {
              "requests" = {
                "storage" = "40Gi"
              }
            }
          }
        }
      }
      "thanos" = {
        "image" = "quay.io/thanos/thanos:v0.36.1"
      }
    }
  }
}
