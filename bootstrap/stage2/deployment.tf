resource "kubernetes_deployment_v1" "operator" {
  wait_for_rollout = true

  metadata {
    namespace = var.namespace
    name      = local.operator_component
    labels = {
      role = local.operator_component
    }
  }

  spec {
    replicas = 1

    // No 2 replicas simultaneously
    strategy {
      type = "Recreate"
    }

    selector {
      match_labels = {
        role = local.operator_component
      }
    }

    template {
      metadata {
        labels = {
          role = local.operator_component
        }
      }

      spec {
        container {
          image   = var.operator_image
          name    = "main"
          command = ["operator"]

          env {
            name  = "K8S_IN_CLUSTER"
            value = "true"
          }

          env {
            name  = "IMAGE"
            value = var.hydra_node_image
          }

          env {
            name  = "OPEN_HEAD_IMAGE"
            value = var.open_head_image
          }

          env {
            name  = "SIDECAR_IMAGE"
            value = var.sidecar_image
          }

          env {
            name  = "REFEREE_IMAGE"
            value = var.referee_image
          }

          env {
            name  = "AI_IMAGE"
            value = var.ai_image
          }

          env {
            name  = "CONFIGMAP"
            value = local.configmap
          }

          env {
            name  = "SECRET"
            value = local.secret
          }

          env {
            name  = "API_KEY"
            value = var.api_key
          }

          env {
            name  = "BLOCKFROST_KEY"
            value = var.blockfrost_key
          }

          env {
            name  = "EXTERNAL_DOMAIN"
            value = var.external_domain
          }

          env {
            name  = "EXTERNAL_PORT"
            value = var.external_port
          }

          env {
            name  = "EXTERNAL_PROTOCOL"
            value = var.external_protocol
          }

          env {
            name  = "ADMIN_ADDR"
            value = var.admin_addr
          }

          env {
            name  = "HYDRA_SCRIPTS_TX_ID"
            value = var.hydra_scripts_tx_id
          }

          env {
            name  = "DMTR_PROJECT_ID"
            value = var.dmtr_project_id
          }

          env {
            name  = "DMTR_API_KEY"
            value = var.dmtr_api_key
          }

          env {
            name  = "DMTR_PORT_NAME"
            value = var.dmtr_port_name
          }

          env {
            name  = "INIT_IMAGE"
            value = var.init_image
          }

          env {
            name  = "BUCKET"
            value = var.bucket
          }

          env {
            name  = "INIT_AWS_ACCESS_KEY_ID"
            value = var.init_aws_access_key_id
          }

          env {
            name  = "INIT_AWS_SECRET_ACCESS_KEY"
            value = var.init_aws_secret_access_key
          }

          resources {
            limits = {
              cpu    = var.resources.limits.cpu
              memory = var.resources.limits.memory
            }
            requests = {
              cpu    = var.resources.requests.cpu
              memory = var.resources.requests.memory
            }
          }

          port {
            name           = "api"
            container_port = 8000
            protocol       = "TCP"
          }
        }

        volume {
          name = "config"
          config_map {
            name = local.configmap
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
