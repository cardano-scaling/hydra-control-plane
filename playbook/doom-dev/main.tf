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

variable "operator_image" {
  type        = string
  description = "The image to use for the operator component."
}

variable "hydra_node_image" {
  type        = string
  description = "The Docker image to use for the Hydra node component."
}

variable "sidecar_image" {
  type        = string
  description = "The Docker image to use for the sidecar component of the Hydra node."
}

variable "dedicated_image" {
  type        = string
  description = "The Docker image to use for the dedicated / referee server of the hydra node."
}

variable "open_head_image" {
  type        = string
  description = "The Docker image to use for the open head component of the Hydra node."
}

variable "control_plane_image" {
  type        = string
  description = "The Docker image to use for the control plane component of the Hydra node."
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

module "stage1" {
  source = "../../bootstrap/stage1/"
}

module "stage2" {
  source     = "../../bootstrap/stage2"
  depends_on = [module.stage1]

  admin_key           = file("${path.module}/admin.sk")
  protocol_parameters = file("${path.module}/protocol-parameters.json")
  external_port       = 80

  namespace           = local.namespace
  external_domain     = var.external_domain
  operator_image      = var.operator_image
  hydra_node_image    = var.hydra_node_image
  sidecar_image       = var.sidecar_image
  dedicated_image     = var.dedicated_image
  open_head_image     = var.open_head_image
  control_plane_image = var.control_plane_image
  blockfrost_key      = var.blockfrost_key
  admin_addr          = var.admin_addr
  dmtr_project_id     = var.dmtr_project_id
  dmtr_api_key        = var.dmtr_api_key
  dmtr_port_name      = var.dmtr_port_name
  hydra_scripts_tx_id = var.hydra_scripts_tx_id
}
