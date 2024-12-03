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
          command           = ["proxy"]

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
            value = "${var.namespace}.svc.cluster.local"
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
    annotations = {
      "service.beta.kubernetes.io/aws-load-balancer-backend-protocol" = "tcp"
      "service.beta.kubernetes.io/aws-load-balancer-nlb-target-type"  = "ip"
      "service.beta.kubernetes.io/aws-load-balancer-scheme"           = "internet-facing"
      "service.beta.kubernetes.io/aws-load-balancer-ssl-cert"         = "${var.ssl_cert_arn}"
      "service.beta.kubernetes.io/aws-load-balancer-ssl-ports"        = "443"
    }
  }

  spec {
    load_balancer_class = "service.k8s.aws/nlb"

    selector = {
      role = "proxy"
    }

    port {
      name        = "proxy"
      port        = 443
      target_port = local.proxy_port
    }

    type = "LoadBalancer"
  }
}
