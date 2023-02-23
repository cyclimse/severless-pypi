import os
import subprocess
import shutil
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


class Package(BaseModel):
    project: str
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


@app.post("/")
async def install_package(package: Package):
    archive = package.archive_url.split("/")[-1]
    if "#" in archive:
        archive = archive.split("#")[0]
    with (
        tempfile.TemporaryDirectory() as build_dir,
        httpx.stream("GET", package.archive_url) as r,
        open(os.path.join(build_dir, archive), "wb") as fp,
    ):
        for chunk in r.iter_bytes():
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

        subprocess.run(
            command,
            check=True,
            cwd=build_dir,
        )

        wheel = None
        for file in pathlib.Path(build_dir).iterdir():
            print(file)
            if file.suffix == ".whl":
                wheel = file.name

        if not wheel:
            raise RuntimeError()

        s3.Bucket(S3_BUCKET).upload_file(
            os.path.join(build_dir, wheel), os.path.join(package.project, wheel)
        )
    return {"message": "Hello World"}
