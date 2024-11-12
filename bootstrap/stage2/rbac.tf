resource "kubernetes_cluster_role" "cluster_role" {
  metadata {
    name = var.namespace
  }

  rule {
    api_groups = ["apps"]
    resources  = ["deployments", "statefulsets"]
    verbs      = ["*"]
  }

  rule {
    api_groups = [""]
    resources  = ["services", "persistentvolumeclaims"]
    verbs      = ["*"]
  }

  rule {
    api_groups = [""]
    resources  = ["configmaps"]
    verbs      = ["*"]
  }

  rule {
    api_groups = ["networking.k8s.io"]
    resources  = ["ingresses"]
    verbs      = ["*"]
  }

  rule {
    api_groups = ["hydra.doom"]
    resources  = ["*"]
    verbs      = ["*"]
  }
}

resource "kubernetes_cluster_role_binding" "cluster_role_binding" {
  metadata {
    name = var.namespace
  }
  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = var.namespace
  }
  subject {
    kind      = "ServiceAccount"
    name      = "default"
    namespace = var.namespace
  }
}
