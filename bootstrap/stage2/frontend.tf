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
          image             = var.frontend_image
          name              = "main"
          image_pull_policy = "Always"

          env {
            name  = "VITE_SERVER_URL"
            value = "https://${local.control_plane_host}/"
          }

          env {
            name  = "VITE_API_BASE_URL"
            value = "https://staging-rewardengine.dripdropz.io/api/v1"
          }

          env {
            name  = "VITE_API_KEY"
            value = "067d20be-8baa-49cb-b501-e004af358870"
          }
          env {
            name  = "VITE_NETWORK_ID"
            value = var.network_id
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
