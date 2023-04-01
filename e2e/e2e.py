import sys
from urllib.parse import urlparse
import shutil
import typing
import tempfile
from pathlib import Path

import docker
from docker.models.containers import Container

import pytest

PYTHON_VERSIONS = ["3.11"]
BASE_IMAGE = "rg.fr-par.scw.cloud/scwfunctionsruntimes-public/python-dep:3.11"

# The tests are run in a container in the same network as the docker-compose services.
DOCKER_COMPOSE_NETWORK = "serverless_pypi_default"
INDEX_URL = "http://index:4000/"


@pytest.fixture()
def cnt_with_tempdir() -> typing.Iterator[typing.Tuple[Container, str]]:
    """A fixture to get a container to run commands in."""
    client = docker.from_env()

    try:
        tempdir = tempfile.mkdtemp()
        container = client.containers.run(
            BASE_IMAGE,
            "sleep infinity",
            detach=True,
            network=DOCKER_COMPOSE_NETWORK,
            remove=True,
            volumes={tempdir: {"bind": "/tmp/e2e", "mode": "rw"}},
        )
        yield (container, tempdir)
    finally:
        container.stop()
        shutil.rmtree(tempdir)


def make_command(package: str) -> list[str]:
    cmd = [
        "pip",
        "install",
        "--timeout",
        "9000",
        "-i",
        INDEX_URL,
        package,
        "--no-cache-dir",
    ]
    # Set index host as trusted
    # Only needed in development because the index is served over HTTP and not HTTPS
    url = urlparse(INDEX_URL)
    cmd.extend(["--trusted-host", url.hostname])
    return cmd


def test_can_install_numpy(cnt_with_tempdir: typing.Tuple[Container, str]):
    cnt, tempdir = cnt_with_tempdir

    _, stream = cnt.exec_run(cmd=make_command("numpy"), stream=True)
    for data in stream:
        sys.stdout.buffer.write(data)

    (Path(tempdir) / "test_np.py").write_text(
        """
import numpy as np
arr = np.array([1, 2, 3, 4, 5])
print(np.sum(arr))
    """
    )

    exit_code, output = cnt.exec_run(cmd="python test_np.py", workdir="/tmp/e2e")

    assert exit_code == 0

    output: str
    res = int(output.rstrip())
    assert res == 15


def test_can_install_pandas(cnt_with_tempdir: typing.Tuple[Container, str]):
    cnt, tempdir = cnt_with_tempdir

    _, stream = cnt.exec_run(cmd=make_command("pandas"), stream=True)
    for data in stream:
        sys.stdout.buffer.write(data)

    (Path(tempdir) / "test_pd.py").write_text(
        """
import pandas as pd
df = pd.DataFrame(
    {
        "Name": [
            "Braund, Mr. Owen Harris",
            "Allen, Mr. William Henry",
            "Bonnell, Miss. Elizabeth",
        ],
        "Age": [22, 35, 58],
        "Sex": ["male", "male", "female"],
    }
)
print(df["Age"].max())
    """
    )

    exit_code, output = cnt.exec_run(cmd="python test_pd.py", workdir="/tmp/e2e")

    assert exit_code == 0

    output: str
    res = int(output.rstrip())
    assert res == 58
