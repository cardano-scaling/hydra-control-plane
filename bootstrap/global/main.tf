variable "monitoring_namespace" {
  type    = string
  default = "hydra-doom-system"
}

variable "thanos_querier_image" {
  type    = string
  default = "quay.io/thanos/thanos:v0.36.1"
}

variable "thanos_endpoints" {
  type = list(string)
}

variable "external_domain" {
  type = string
}
