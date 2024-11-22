resource "kubernetes_ingress_v1" "grafana_ingress" {
  metadata {
    name      = "grafana"
    namespace = var.monitoring_namespace
  }

  spec {
    ingress_class_name = "nginx"
    rule {
      host = "grafana.${var.external_domain}"
      http {
        path {
          path      = "/"
          path_type = "Prefix"
          backend {
            service {
              name = "grafana"
              port {
                number = 3000
              }
            }
          }
        }
      }
    }
  }
}
