resource "kubernetes_deployment_v1" "proxy" {
  wait_for_rollout = true

  metadata {
    name      = "proxy"
    namespace = var.namespace
    labels = {
      role = "proxy"
    }
  }
  spec {
    replicas = var.proxy_replicas
    selector {
      match_labels = {
        role = "proxy"
      }
    }
    template {
      metadata {
        name = "proxy"
        labels = {
          role = "proxy"
        }
      }
      spec {
        container {
          name              = "main"
          image             = var.proxy_image
          image_pull_policy = "IfNotPresent"

          resources {
            limits = {
              cpu    = var.proxy_resources.limits.cpu
              memory = var.proxy_resources.limits.memory
            }
            requests = {
              cpu    = var.proxy_resources.requests.cpu
              memory = var.proxy_resources.requests.memory
            }
          }

          port {
            name           = "proxy"
            container_port = local.proxy_port
            protocol       = "TCP"
          }

          env {
            name  = "PROXY_ADDR"
            value = local.proxy_addr
          }

          env {
            name  = "HYDRA_NODE_PORT"
            value = 4001
          }

          env {
            name  = "HYDRA_NODE_DNS"
            value = "svc.cluster.local"
          }
        }
      }
    }
  }
}

resource "kubernetes_service_v1" "proxy_service" {
  metadata {
    name      = "proxy"
    namespace = var.namespace
  }

  spec {
    type = "ClusterIP"

    selector = {
      role = "proxy"
    }

    port {
      name        = "proxy"
      port        = local.proxy_port
      target_port = local.proxy_port
    }
  }
}
