resource "kubernetes_deployment_v1" "discord_bot" {
  wait_for_rollout = true

  metadata {
    namespace = var.monitoring_namespace
    name      = "discord-bot"
    labels = {
      role = "discord-bot"
    }
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        role = "discord-bot"
      }
    }

    template {
      metadata {
        labels = {
          role = "discord-bot"
        }
      }

      spec {
        container {
          image             = var.discord_bot_image
          name              = "main"
          image_pull_policy = "Always"

          env {
            name  = "TOKEN"
            value = var.discord_bot_token
          }

          env {
            name  = "OWNER_ID"
            value = var.discord_bot_owner_id
          }

          env {
            name  = "ADMIN_ROLE_ID"
            value = var.discord_bot_admin_role_id
          }

          env {
            name  = "CHANNEL_ID"
            value = var.discord_bot_channel_id
          }

          resources {
            limits = {
              memory = "1Gi"
            }
            requests = {
              cpu    = "200m"
              memory = "1Gi"
            }
          }
        }
      }
    }
  }
}
