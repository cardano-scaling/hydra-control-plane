resource "kubernetes_config_map" "node-config" {
  metadata {
    namespace = var.namespace
    name      = local.configmap
  }

  data = {
    "admin.sk"                 = var.admin_key
    "protocol-parameters.json" = var.protocol_parameters
  }
}
