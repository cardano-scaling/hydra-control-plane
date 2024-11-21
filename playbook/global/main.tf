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
    "k8s-hydradoo-thanossi-3e6cc6bace-ddd76e7d5e148d9f.elb.us-east-1.amazonaws.com:10901",
    "k8s-hydradoo-thanossi-08d03cf670-c832566453f2a5a0.elb.eu-central-1.amazonaws.com:10901",
  ]
}
