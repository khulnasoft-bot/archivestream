variable "region" {
  description: "AWS region"
  type        : string
  default     : "us-east-1"
}

variable "cluster_name" {
  description: "EKS cluster name"
  type        : string
  default     : "archivestream-prod"
}

variable "db_password" {
  description: "PostgreSQL admin password"
  type        : string
  sensitive   : true
}
