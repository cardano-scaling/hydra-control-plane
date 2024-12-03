resource "kubernetes_config_map" "node-config" {
  metadata {
    namespace = var.namespace
    name      = local.configmap
  }

  data = {
    "protocol-parameters.json" = var.protocol_parameters
    "shelley-genesis.json"     = var.shelley_genesis
  }
}
