locals {
  namespace = "hydra-doom"
}

terraform {
  backend "s3" {
    bucket = "hydra-doom-tf"
    key    = "clusters/hydra-doom-dev-cluster/tfstate"
    region = "us-east-1"
  }
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "2.22.0"
    }
  }
}

resource "kubernetes_namespace" "namespace" {
  metadata {
    name = local.namespace
  }
}

variable "blockfrost_key" {
  type = string
}

variable "dmtr_project_id" {
  type = string
}

variable "dmtr_api_key" {
  type = string
}

variable "dmtr_port_name" {
  type = string
}

variable "external_domain" {
  type        = string
  description = "The domain prefix that will be used to access the hydra node."
}

variable "image" {
  type = string
}

variable "hydra_node_image" {
  type        = string
  description = "The Docker image to use for the Hydra node component."
}

variable "hydra_scripts_tx_id" {
  type        = string
  description = "The transaction ID of the Hydra scripts."
}

variable "admin_addr" {
  type        = string
  description = "The address of the admin key."
}

variable "eks_cluster_arn" {
  type        = string
  description = "The ARN of the EKS cluster."
}

variable "admin_key" {
  type = string
}

variable "frontend_image" {
  type = string
}

variable "frontend_replicas" {
  type    = number
  default = 1
}

provider "kubernetes" {
  config_path    = "~/.kube/config"
  config_context = var.eks_cluster_arn
}

provider "helm" {
  kubernetes {
    config_path    = "~/.kube/config"
    config_context = var.eks_cluster_arn
  }
}

module "stage2" {
  source = "../../bootstrap/stage2"

  admin_key           = var.admin_key
  protocol_parameters = file("${path.module}/protocol-parameters.json")
  external_port       = 443
  external_protocol   = "wss"

  namespace           = local.namespace
  external_domain     = var.external_domain
  hydra_node_image    = var.hydra_node_image
  operator_image      = var.image
  sidecar_image       = var.image
  open_head_image     = var.image
  control_plane_image = var.image
  blockfrost_key      = var.blockfrost_key
  admin_addr          = var.admin_addr
  dmtr_project_id     = var.dmtr_project_id
  dmtr_api_key        = var.dmtr_api_key
  dmtr_port_name      = var.dmtr_port_name
  hydra_scripts_tx_id = var.hydra_scripts_tx_id
  frontend_image      = var.frontend_image
  frontend_replicas   = var.frontend_replicas
}
