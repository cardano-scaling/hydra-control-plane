terraform {
  backend "s3" {
    bucket = "hydra-doom-tf"
    key    = "clusters/hydra-doom-dev-cluster/tfstate.global"
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

provider "kubernetes" {
  config_path    = "~/.kube/config"
  config_context = var.eks_cluster_arn
}

module "global" {
  source = "../../bootstrap/global/"

  thanos_endpoints = [
    "k8s-hydradoo-thanossi-806f9000b2-386099bd8a7733a9.elb.us-east-1.amazonaws.com:10901",
    "k8s-hydradoo-thanossi-2c4000794d-85ac7f5b7a39c8f7.elb.eu-central-1.amazonaws.com:10901",
  ]
}
