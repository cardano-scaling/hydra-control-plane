locals {
  node_arguments = [
    "run",
    "--database-path",
    "/data/db",
    "--socket-path",
    "/ipc/node.socket",
    "--port",
    "3000"
  ]
  n2n_port_name = "n2n"
  network       = var.network_id == 0 ? "preprod" : "mainnet"
  magic         = var.network_id == 0 ? 1 : 764824073
}

resource "kubernetes_config_map" "node_proxy_config" {
  metadata {
    namespace = var.namespace
    name      = "proxy-${local.network}"
  }

  data = {
    "nginx.conf" = "${file("${path.module}/nginx.conf")}"
  }
}

resource "kubernetes_stateful_set_v1" "node" {
  wait_for_rollout = false

  metadata {
    namespace = var.namespace
    name      = "node-${local.network}"
    labels = {
      network = local.network
      role    = "cardano-node"
    }
  }

  spec {
    replicas     = var.node_replicas
    service_name = "nodes-${local.network}"

    selector {
      match_labels = {
        network = local.network
        role    = "cardano-node"
      }
    }

    volume_claim_template {
      metadata {
        name = "data"
      }
      spec {
        access_modes       = ["ReadWriteOnce"]
        storage_class_name = "gp2"
        resources {
          requests = {
            storage = "220Gi"
          }
        }
      }
    }

    template {
      metadata {
        labels = {
          network = local.network
          role    = "cardano-node"
        }
      }

      spec {
        volume {
          name = "ipc"
          empty_dir {}
        }

        volume {
          name = "proxy-config"
          config_map {
            name = "proxy-${local.network}"
          }
        }

        container {
          image = "${var.node_image}:${var.node_image_tag}"
          name  = "main"

          args = local.node_arguments

          env {
            name  = "CARDANO_NETWORK"
            value = local.network
          }

          env {
            name  = "RESTORE_SNAPSHOT"
            value = "true"
          }

          env {
            name  = "CARDANO_NODE_SOCKET_PATH"
            value = "/ipc/node.socket"
          }

          env {
            name  = "CARDANO_NODE_NETWORK_ID"
            value = local.magic
          }

          resources {
            limits   = var.node_resources.limits
            requests = var.node_resources.requests
          }

          port {
            name           = local.n2n_port_name
            container_port = 3000
          }

          port {
            name           = "metrics"
            container_port = 12798
          }

          volume_mount {
            mount_path = "/data"
            name       = "data"
          }

          volume_mount {
            mount_path = "/ipc"
            name       = "ipc"
          }
        }

        container {
          name  = "nginx"
          image = "nginx"

          resources {
            limits = {
              memory = "100Mi"
            }
            requests = {
              cpu    = "10m"
              memory = "100Mi"
            }
          }

          port {
            name           = "n2c"
            container_port = 3307
          }

          volume_mount {
            mount_path = "/ipc"
            name       = "ipc"
          }

          volume_mount {
            mount_path = "/etc/nginx"
            name       = "proxy-config"
          }
        }
      }
    }
  }
}

resource "kubernetes_service_v1" "node" {
  metadata {
    name      = "node-${local.network}"
    namespace = var.namespace
  }

  spec {
    port {
      name     = "n2c"
      protocol = "TCP"
      port     = 3307
    }

    port {
      name     = "n2n"
      protocol = "TCP"
      port     = 3000
    }

    selector = {
      "role"    = "cardano-node"
      "network" = local.network
    }

    type = "ClusterIP"
  }
}
