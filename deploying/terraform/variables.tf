variable "bucket_name" {
  type        = string
  description = "Bucket to store dependencies into."
  default     = "sls-pypi-index"
}

variable "zig_toolchain" {
  type        = bool
  description = "Use a Zig-based compilation toolchain"
  default     = true
}

variable "worker_pyhon_version" {
  type        = string
  description = "Python version to use for the worker image."
  default     = "3.11"
}

variable "worker_memory_limit" {
  type        = number
  description = "Memory limit to use for worker. Higher makes builds faster."
  default     = 4096
}
