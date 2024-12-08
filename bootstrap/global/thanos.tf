resource "kubernetes_deployment_v1" "thanos_querier" {
  wait_for_rollout = false

  metadata {
    namespace = var.monitoring_namespace
    name      = "thanos-querier"
    labels = {
      role = "thanos-querier"
    }
  }

  spec {
    replicas = 2

    selector {
      match_labels = {
        role = "thanos-querier"
      }
    }

    template {
      metadata {
        labels = {
          role = "thanos-querier"
        }
      }

      spec {
        container {
          image = var.thanos_querier_image
          name  = "thanos"
          args = concat([
            "query",
            "--log.level=debug",
            "--query.replica-label=replica",
          ], [for endpoint in var.thanos_endpoints : "--endpoint=${endpoint}"])

          port {
            name           = "http"
            container_port = 10902
          }

          port {
            name           = "grpc"
            container_port = 10901
          }

          liveness_probe {
            http_get {
              path = "/-/healthy"
              port = "http"
            }
          }
        }
      }
    }
  }
}

resource "kubernetes_service_v1" "thanos_querier" {
  metadata {
    name      = "thanos-querier"
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
      role = "thanos-querier"
    }

    port {
      name        = "http"
      port        = 443
      target_port = "http"
      protocol    = "TCP"
    }
  }
}
