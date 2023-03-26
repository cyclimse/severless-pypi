provider "docker" {
  host = "unix:///var/run/docker.sock"

  registry_auth {
    address  = scaleway_container_namespace.main.registry_endpoint
    username = "nologin"
    password = scaleway_iam_api_key.registry_push.secret_key
  }
}

resource "docker_image" "index" {
  name = "${scaleway_container_namespace.main.registry_endpoint}/index:latest"
  build {
    context = "${path.cwd}/../.."
  }

  provisioner "local-exec" {
    command = "docker push ${docker_image.index.name}"
  }
}

resource "docker_image" "worker" {
  name = "${scaleway_container_namespace.main.registry_endpoint}/worker:latest"
  build {
    context = "${path.cwd}/../../worker"
    build_args = {
      PYTHON_VERSION = var.worker_pyhon_version
    }
  }

  provisioner "local-exec" {
    command = "docker push ${docker_image.worker.name}"
  }
}
