resource "kubernetes_ingress_v1" "ingress" {
  metadata {
    name      = "hydra-doom"
    namespace = var.namespace
  }

  spec {
    ingress_class_name = "nginx"

    rule {
      host = "${var.frontend_prefix}.${var.external_domain}"
      http {
        path {
          path      = "/"
          path_type = "Prefix"
          backend {
            service {
              name = local.frontend_component
              port {
                number = local.frontend_port
              }
            }
          }
        }
      }
    }

    rule {
      host = local.control_plane_host
      http {
        path {
          path      = "/"
          path_type = "Prefix"
          backend {
            service {
              name = local.control_plane_component
              port {
                number = 8000
              }
            }
          }
        }
      }
    }

    // Rest are assumed to be nodes, handled by proxy
    rule {
      host = "*.${var.external_domain}"
      http {
        path {
          path      = "/"
          path_type = "Prefix"
          backend {
            service {
              name = "proxy"
              port {
                number = 443
              }
            }
          }
        }
      }
    }
  }
}
