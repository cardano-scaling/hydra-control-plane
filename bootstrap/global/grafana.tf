resource "kubernetes_service_v1" "grafana_lb" {
  metadata {
    name      = "grafana-lb"
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
      "app.kubernetes.io/component" = "o11y"
      "app.kubernetes.io/name"      = "grafana"
      "app.kubernetes.io/part-of"   = "hydradoom"
    }

    port {
      name        = "http"
      port        = 443
      target_port = 3000
      protocol    = "TCP"
    }
  }
}
