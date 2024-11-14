resource "kubernetes_deployment_v1" "control_plane" {
  wait_for_rollout = true

  metadata {
    namespace = var.namespace
    name      = local.control_plane_component
    labels = {
      role = local.control_plane_component
    }
  }

  spec {
    // Avoid race conditions
    replicas = 1

    // No 2 replicas simultaneously
    strategy {
      type = "Recreate"
    }

    selector {
      match_labels = {
        role = local.control_plane_component
      }
    }

    template {
      metadata {
        labels = {
          role = local.control_plane_component
        }
      }

      spec {
        container {
          image = var.control_plane_image
          name  = "main"

          command = ["rpc"]

          env {
            name  = "K8S_IN_CLUSTER"
            value = "true"
          }

          env {
            name  = "ROCKET_LOG_LEVEL"
            value = "normal"
          }

          env {
            name  = "ROCKET_ADDRESS"
            value = "0.0.0.0"
          }

          env {
            name  = "ROCKET_PORT"
            value = 8000
          }

          env {
            name  = "ROCKET_ADMIN_KEY_FILE"
            value = "${local.secret_mount_path}/admin.sk"
          }

          env {
            name = "KUBERNETES_NAMESPACE"
            value_from {
              field_ref {
                field_path = "metadata.namespace"
              }
            }
          }

          volume_mount {
            name       = "secret"
            mount_path = local.secret_mount_path
          }

          resources {
            limits = {
              cpu    = var.control_plane_resources.limits.cpu
              memory = var.control_plane_resources.limits.memory
            }
            requests = {
              cpu    = var.control_plane_resources.requests.cpu
              memory = var.control_plane_resources.requests.memory
            }
          }

          port {
            name           = "api"
            container_port = 8000
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

resource "kubernetes_service_v1" "control_plane_service" {
  metadata {
    name      = local.control_plane_component
    namespace = var.namespace
  }

  spec {
    type = "ClusterIP"

    selector = {
      role = local.control_plane_component
    }

    port {
      name        = "api"
      port        = 8000
      target_port = 8000
    }
  }
}

resource "kubernetes_ingress_v1" "control_plane_ingress" {
  metadata {
    name      = local.control_plane_component
    namespace = var.namespace
  }

  spec {
    ingress_class_name = "nginx"
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
  }
}
