variable "ssl_cert_arn" {
  type = string
}

variable "namespace" {
  type    = string
  default = "hydra-doom-system"
}

variable "storage_class" {
  description = "storage class name to use for workload PVCs"
  default     = "gp2"
}

variable "cluster_name" {
  description = "Name of the cluster, used as label for prometheus."
}

variable "network_id" {
  type    = number
  default = 1 # mainnet
}

variable "node_image" {
  type    = string
  default = "ghcr.io/blinklabs-io/cardano-node"
}

variable "node_image_tag" {
  type    = string
  default = "10.1.3"
}

variable "node_replicas" {
  type    = number
  default = 1
}

variable "node_resources" {
  type = object({
    requests = map(string)
    limits   = map(string)
  })
  default = {
    limits = {
      "memory" = "22Gi"
      "cpu"    = "8"
    }
    requests = {
      "memory" = "22Gi"
      "cpu"    = "2"
    }
  }
}

resource "kubernetes_namespace_v1" "namespace" {
  metadata {
    name = var.namespace
  }
}

module "o11y_requirements" {
  depends_on = [kubernetes_namespace_v1.namespace]
  source     = "./o11y/requirements"
  namespace  = var.namespace
}

module "o11y_resources" {
  depends_on    = [module.o11y_requirements]
  source        = "./o11y/resources"
  namespace     = var.namespace
  storage_class = var.storage_class
  cluster_name  = var.cluster_name
}
