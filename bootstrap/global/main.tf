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

variable "ssl_cert_arn" {
  type = string
}

variable "discord_bot_image" {
  description = "Docker image for the Discord bot"
  type        = string
}

variable "discord_bot_token" {
  description = "Authentication token for the Discord bot"
  type        = string
  sensitive   = true
}

variable "discord_bot_owner_id" {
  description = "Discord user ID of the bot owner"
  type        = string
}

variable "discord_bot_channel_id" {
  description = "Discord channel ID where the bot will operate"
  type        = string
}

variable "discord_bot_admin_role_id" {
  description = "Discord role ID for bot admins"
  type        = string
}
