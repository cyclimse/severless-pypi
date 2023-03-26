data "scaleway_account_project" "default" {
  name = "default"
}

resource "scaleway_iam_application" "index" {
  name = "pypi-index-app"
}

resource "scaleway_iam_policy" "object_read_only" {
  name           = "object-read-only"
  description    = "Give index read access to s3 bucket."
  application_id = scaleway_iam_application.index.id
  rule {
    project_ids          = [data.scaleway_account_project.default.id]
    permission_set_names = ["ObjectStorageFullAccess"]
  }
}

resource "scaleway_iam_api_key" "index" {
  application_id = scaleway_iam_application.index.id
  description    = "API key for PyPI index."
}

resource "scaleway_iam_application" "worker" {
  name = "pypi-worker-app"
}

resource "scaleway_iam_policy" "object_read_write" {
  name           = "object-read-write"
  description    = "Give worker read-write access to bucket."
  application_id = scaleway_iam_application.worker.id
  rule {
    project_ids          = [data.scaleway_account_project.default.id]
    permission_set_names = ["ObjectStorageFullAccess"]
  }
}

resource "scaleway_iam_api_key" "worker" {
  application_id = scaleway_iam_application.worker.id
  description    = "API key for PyPI worker."
}

resource "scaleway_iam_application" "registry_push" {
  name = "pypi-registry-push-app"
}

resource "scaleway_iam_policy" "registry_full_access" {
  name           = "registry-full-access"
  description    = "Give full access to container registry."
  application_id = scaleway_iam_application.registry_push.id
  rule {
    project_ids          = [data.scaleway_account_project.default.id]
    permission_set_names = ["ContainerRegistryFullAccess"]
  }
}

resource "scaleway_iam_api_key" "registry_push" {
  application_id = scaleway_iam_application.registry_push.id
  description    = "Ephemeral API key to push to registry."

  expires_at = timeadd(timestamp(), "1h")
}
