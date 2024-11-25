locals {
  operator_component      = "operator"
  configmap               = "hydra-pod-config"
  secret                  = "hydra-pod-admin-key"
  secret_mount_path       = "/var/secret"
  control_plane_component = "control-plane"
  control_plane_host      = "${var.control_plane_prefix}.${var.external_domain}"
  frontend_component      = "frontend"
  frontend_port           = 3000
}

variable "namespace" {
  type    = string
  default = "hydra-doom"
}

variable "operator_image" {
  type = string
}

variable "hydra_node_image" {
  type    = string
  default = "ghcr.io/cardano-scaling/hydra-node"
}

variable "open_head_image" {
  type = string
}

variable "sidecar_image" {
  type = string
}

variable "referee_image" {
  type = string
}

variable "ai_image" {
  type = string
}

variable "control_plane_image" {
  type = string
}

variable "blockfrost_key" {
  type = string
}

variable "external_domain" {
  type = string
}

variable "control_plane_prefix" {
  type    = string
  default = "api"
}

variable "frontend_prefix" {
  type    = string
  default = "frontend"
}

variable "frontend_image" {
  type = string
}

variable "frontend_replicas" {
  type    = number
  default = 2
}

variable "external_port" {
  type = number
}

variable "external_protocol" {
  type = string
}

variable "admin_key" {
  type        = string
  description = "The admin key in cardano-cli JSON format."
}

variable "protocol_parameters" {
  type        = string
  description = "The protocol parameters in JSON format."
}

variable "admin_addr" {
  type        = string
  description = "Must be consistent with admin key, calculated using cardano cli."
}

variable "hydra_scripts_tx_id" {
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

variable "init_image" {
  type = string
}

variable "bucket" {
  type    = string
  default = "hydradoomsnapshots"
}

variable "bucket_region" {
  type    = string
  default = "us-east-1"
}

variable "init_aws_access_key_id" {
  type = string
}

variable "init_aws_secret_access_key" {
  type = string
}

variable "api_key" {
  type = string
}

variable "autoscaler_region_prefix" {
  type = string
}

variable "autoscaler_low_watermark" {
  type    = number
  default = 1
}

variable "autoscaler_high_watermark" {
  type    = number
  default = 5
}

variable "autoscaler_max_batch" {
  type = number
}

variable "tolerations" {
  type = list(object({
    effect   = string
    key      = string
    operator = string
    value    = optional(string)
  }))
  default = []
}

variable "resources" {
  type = object({
    limits = object({
      cpu    = optional(string)
      memory = string
    })
    requests = object({
      cpu    = string
      memory = string
    })
  })
  default = {
    requests = {
      cpu    = "500m"
      memory = "512Mi"
    }
    limits = {
      cpu    = "2"
      memory = "512Mi"
    }
  }
}

variable "control_plane_resources" {
  type = object({
    limits = object({
      cpu    = optional(string)
      memory = string
    })
    requests = object({
      cpu    = string
      memory = string
    })
  })
  default = {
    requests = {
      cpu    = "500m"
      memory = "512Mi"
    }
    limits = {
      cpu    = "2"
      memory = "512Mi"
    }
  }
}

variable "frontend_resources" {
  type = object({
    limits = object({
      cpu    = optional(string)
      memory = string
    })
    requests = object({
      cpu    = string
      memory = string
    })
  })
  default = {
    requests = {
      cpu    = "500m"
      memory = "512Mi"
    }
    limits = {
      cpu    = "2"
      memory = "512Mi"
    }
  }
}
