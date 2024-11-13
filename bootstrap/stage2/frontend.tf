resource "kubernetes_deployment_v1" "frontend" {
  wait_for_rollout = true

  metadata {
    namespace = var.namespace
    name      = local.frontend_component
    labels = {
      role = local.frontend_component
    }
  }

  spec {
    replicas = var.frontend_replicas

    selector {
      match_labels = {
        role = local.frontend_component
      }
    }

    template {
      metadata {
        labels = {
          role = local.frontend_component
        }
      }

      spec {
        container {
          image = var.frontend_image
          name  = "main"

          env {
            name  = "REGION"
            value = var.frontend_region
          }

          env {
            name  = "SERVER_URL"
            value = "http://${local.control_plane_url}:80"
          }

          env {
            name  = "CABINET_KEY"
            value = var.frontend_cabinet_key
          }

          env {
            name  = "PERSISTENT_SESSION"
            value = var.frontend_persistent_session
          }

          volume_mount {
            name       = "secret"
            mount_path = local.secret_mount_path
          }

          resources {
            limits = {
              cpu    = var.frontend_resources.limits.cpu
              memory = var.frontend_resources.limits.memory
            }
            requests = {
              cpu    = var.frontend_resources.requests.cpu
              memory = var.frontend_resources.requests.memory
            }
          }

          port {
            name           = "api"
            container_port = local.frontend_port
            protocol       = "TCP"
          }
        }

        volume {
          name = "secret"
          secret {
            secret_name = local.secret
          }
        }

        dynamic "toleration" {
          for_each = var.tolerations

          content {
            effect   = toleration.value.effect
            key      = toleration.value.key
            operator = toleration.value.operator
            value    = toleration.value.value
          }
        }
      }
    }
  }
}

resource "kubernetes_service_v1" "frontend_service" {
  metadata {
    name      = local.frontend_component
    namespace = var.namespace
  }

  spec {
    type = "ClusterIP"

    selector = {
      role = local.frontend_component
    }

    port {
      name        = "api"
      port        = local.frontend_port
      target_port = local.frontend_port
    }
  }
}

resource "kubernetes_ingress_v1" "frontend_ingress" {
  metadata {
    name      = local.frontend_component
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
  }
}
