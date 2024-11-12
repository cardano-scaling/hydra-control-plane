resource "kubernetes_secret" "admin_key" {
  metadata {
    name      = local.secret
    namespace = var.namespace
  }
  data = {
    "admin.sk" = var.admin_key
  }
  type = "Opaque"
}
