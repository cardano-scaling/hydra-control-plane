locals {
  namespace = "hydra-doom"
}

terraform {
  backend "s3" {
    bucket = "hydra-doom-tf"
    region = "us-east-1"
  }
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "2.22.0"
    }
  }
}

variable "eks_cluster_arn" {
  type        = string
  description = "The ARN of the EKS cluster."
}
variable "ssl_cert_arn" {
  type = string
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
  source       = "../../bootstrap/stage1/"
  ssl_cert_arn = var.ssl_cert_arn
}
