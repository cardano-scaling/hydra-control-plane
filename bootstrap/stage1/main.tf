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
