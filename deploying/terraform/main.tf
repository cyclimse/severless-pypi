resource "scaleway_object_bucket" "main" {
  name = var.bucket_name
}

resource "scaleway_container_namespace" "main" {
  name        = "pypi-index"
  description = "Application to package native dependencies for Python functions."
}

locals {
  bucket_endpoint = "https://s3.${scaleway_object_bucket.main.region}.scw.cloud"
  timeout         = 900 // 15 minutes which is the maximum
}

resource "scaleway_container" "index" {
  name           = "pypi-index"
  description    = "Serverless PyPI index."
  namespace_id   = scaleway_container_namespace.main.id
  registry_image = docker_image.index.name
  port           = 4000
  privacy        = "public"
  deploy         = true
  timeout        = local.timeout

  environment_variables = {
    S3_BUCKET          = var.bucket_name
    S3_ENDPOINT        = local.bucket_endpoint
    SCW_DEFAULT_REGION = scaleway_object_bucket.main.region
    PYPI_INDEX         = "pypi.org"
    WORKER_URL         = "https://${scaleway_container.worker.domain_name}"
  }
  secret_environment_variables = {
    SCW_ACCESS_KEY = scaleway_iam_api_key.index.access_key
    SCW_SECRET_KEY = scaleway_iam_api_key.index.secret_key
  }
}

resource "scaleway_container" "worker" {
  name           = "pypi-worker"
  description    = "Serverless PyPI worker."
  namespace_id   = scaleway_container_namespace.main.id
  registry_image = docker_image.worker.name
  port           = 8080
  privacy        = "public"
  deploy         = true
  timeout        = local.timeout

  memory_limit = var.worker_memory_limit

  environment_variables = {
    S3_BUCKET     = var.bucket_name
    S3_ENDPOINT   = local.bucket_endpoint
    S3_REGION     = scaleway_object_bucket.main.region
    ZIG_TOOLCHAIN = var.zig_toolchain ? "yes" : ""
  }
  secret_environment_variables = {
    SCW_ACCESS_KEY = scaleway_iam_api_key.worker.access_key
    SCW_SECRET_KEY = scaleway_iam_api_key.worker.secret_key
  }
}
