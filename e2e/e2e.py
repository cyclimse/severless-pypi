import sys
from urllib.parse import urlparse

import docker
import pytest

PYTHON_VERSIONS = ["3.10", "3.11"]
BASE_IMAGE = "rg.fr-par.scw.cloud/scwfunctionsruntimes-public/python-dep:"

# The tests are run in a container in the same network as the docker-compose services.
DOCKER_COMPOSE_NETWORK = "serverless_pypi_default"
INDEX_URL = "http://index:4000/"


@pytest.fixture(params=PYTHON_VERSIONS)
def alpine_container(request):
    """A fixture to get a container to run commands in."""
    client = docker.from_env()

    image = BASE_IMAGE + request.param
    try:
        container = client.containers.run(
            image,
            "sleep infinity",
            detach=True,
            network=DOCKER_COMPOSE_NETWORK,
            remove=True,
        )
        yield container
    finally:
        container.stop()


def make_command(package: str) -> list[str]:
    cmd = ["pip", "install", "-i", INDEX_URL, package, "--no-cache-dir"]
    # Set index host as trusted
    # Only needed in development because the index is served over HTTP and not HTTPS
    url = urlparse(INDEX_URL)
    cmd.extend(["--trusted-host", url.hostname])
    return cmd


def test_can_install_dagon(alpine_container):
    _, stream = alpine_container.exec_run(cmd=make_command("dagon"), stream=True)
    for data in stream:
        sys.stdout.buffer.write(data)
