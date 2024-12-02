resource "kubernetes_manifest" "pod_monitors" {
  manifest = {
    apiVersion = "monitoring.coreos.com/v1"
    kind       = "PodMonitor"
    metadata = {
      labels = {
        "app.kubernetes.io/component" = "o11y"
        "app.kubernetes.io/part-of"   = "hydradoom"
      }
      name      = "hydradoomnodes"
      namespace = var.namespace
    }
    spec = {
      selector = {
        matchLabels = {
          component = "hydra-doom-node"
        }
      }
      podMetricsEndpoints = [
        {
          port = "metrics",
          path = "/metrics",
          relabelings = [
            {
              action = "labeldrop"
              regex  = "(container|endpoint|instance|job|namespace|prometheus|prometheus_replica)"
            }
          ],
          metricRelabelings = [
            {
              action = "labeldrop"
              regex  = "(container|endpoint|instance|job|namespace|prometheus|prometheus_replica)"
            }
          ]
        }
      ]
    }
  }
}
