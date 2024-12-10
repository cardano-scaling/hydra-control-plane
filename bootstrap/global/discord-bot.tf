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

resource "kubernetes_service_v1" "discord_bot_lb" {
  metadata {
    name      = "discord-bot-lb"
    namespace = var.monitoring_namespace
    annotations = {
      "service.beta.kubernetes.io/aws-load-balancer-nlb-target-type" : "instance"
      "service.beta.kubernetes.io/aws-load-balancer-scheme" : "internet-facing"
      "service.beta.kubernetes.io/aws-load-balancer-type" : "external"
      "service.beta.kubernetes.io/aws-load-balancer-ssl-cert" : var.ssl_cert_arn
      "service.beta.kubernetes.io/aws-load-balancer-ssl-ports" : "443"
    }
  }

  spec {
    type = "LoadBalancer"

    load_balancer_class = "service.k8s.aws/nlb"
    selector = {
      role = "discord-bot"
    }

    port {
      name        = "http"
      port        = 443
      target_port = 8080
      protocol    = "TCP"
    }
  }
}
