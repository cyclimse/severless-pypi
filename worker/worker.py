import os
import logging
import subprocess
import tempfile
import sys
import pathlib

import boto3
from fastapi import FastAPI
import httpx
from pydantic import BaseModel

SCW_ACCESS_KEY = os.environ["SCW_ACCESS_KEY"]
SCW_SECRET_KEY = os.environ["SCW_SECRET_KEY"]
S3_BUCKET = os.environ["S3_BUCKET"]
S3_ENDPOINT = os.getenv("S3_ENDPOINT", "https://s3.fr-par.scw.cloud")
S3_REGION = os.getenv("S3_REGION", "fr-par")
ZIG_TOOLCHAIN = os.getenv("ZIG_TOOLCHAIN", "")

logging.basicConfig(format="{levelname:7} {message}", style="{", level=logging.INFO)


class Package(BaseModel):
    project: str
    filename: str
    archive_url: str


s3 = boto3.resource(
    "s3",
    region_name="fr-par",
    use_ssl=True,
    endpoint_url=S3_ENDPOINT,
    aws_access_key_id=SCW_ACCESS_KEY,
    aws_secret_access_key=SCW_SECRET_KEY,
)

app = FastAPI()


def get_zig_toolchain_env() -> dict[str, str]:
    """Set the environment variables to use Zig."""
    return {
        "CC": "zig cc",
        "CXX": "zig c++",
        "CFLAGS": "-mtune=x86_64 -lc++ -Os -g0 -ftls-model=global-dynamic -Wl,--strip--all",
    }


@app.post("/")
async def install_package(package: Package):
    archive = package.archive_url.split("/")[-1]
    if "#" in archive:
        archive = archive.split("#")[0]
    with (
        tempfile.TemporaryDirectory() as build_dir,
        httpx.stream("GET", package.archive_url) as req,
        open(os.path.join(build_dir, archive), "wb") as fp,
    ):
        for chunk in req.iter_bytes():
            fp.write(chunk)
        fp.flush()

        python_path = sys.executable
        command = [
            python_path,
            "-m",
            "pip",
            "wheel",
            "--no-deps",
            os.path.join(build_dir, archive),
        ]

        env = {}
        if ZIG_TOOLCHAIN:
            env |= get_zig_toolchain_env()

        subprocess.run(
            command,
            check=True,
            cwd=build_dir,
            env=env,
        )

        logging.info(
            "Successfully executed command %s",
            " ".join(command[2:]),
        )

        # TODO?: cleanup this
        wheel = None
        for file in pathlib.Path(build_dir).iterdir():
            if file.suffix == ".whl":
                wheel = file.name

        if not wheel:
            raise RuntimeError("Could not find wheel in output directory!")

        key = os.path.join(package.project, package.filename)
        logging.info(
            "Uploading wheel %s to bucket %s with key %s", wheel, S3_BUCKET, key
        )

        s3.Bucket(S3_BUCKET).upload_file(
            os.path.join(build_dir, wheel),
            key,
        )

    return {"message": f"Successfully built {package.filename}"}
