# Serverless PyPI Builder

An experimental PyPI dependency builder for Scaleway Serverless Python functions.

This project is aimed at simplifying the use of native libraries with Scaleway Python functions. Instead of building  your native dependencies on your laptop, you can deploy this project and build them on the public Cloud directly.

It uses two containers on `min-scale=0` so you will only pay when running builds.

You can learn more about compiling native dependencies for Scaleway Serverless Python functions on the [Scaleway Documentation](https://www.scaleway.com/en/docs/serverless/functions/how-to/package-function-dependencies-in-zip/#additional-dependencies).

> **Note**
> Not an official project of the Serverless Team

## Quickstart

You can deploy this project on Scaleway with Terraform:

```console
cd deploying/terraform/

terraform init
terraform apply
```

This will create two containers and an S3 bucket. One container will be your PyPI index, and another will build the packages.

Once everything is deployed, you can use the index container as PyPI mirror:

```console
export MY_INDEX=$(terraform output -raw index_url)

pip install --timeout=900 -i $MY_INDEX numpy --target ./package
```

By default, packages are compiled with `-Os` and stripped. If you need to further reduce the size, you can remove `.pyc` and test files from your `package` directory:

```console
find . -name \*.pyc -delete

# Additionnaly remove test files
# This can save a lot of space

find . -name test_\*.py -delete
```

## Zig toolchain

By default, the compilation of C libraries is done with [Zig](https://zig.news/kristoff/compile-a-c-c-project-with-zig-368j). From my own biased testing, the compilation is a bit faster with Zig than gcc.

## Known issues

### Timeout

If a build exceeds 15 minutes, it will fail. Sometimes, the wheel will still get pushed to s3 after a timeout. In this case, running pip again may allow you to install your dependency anyways.

### Python versions

Because of the way this works, it's important to have the same Python version between the worker and the runtime of your Python functions. This can be configured in Terraform via the `worker_python_version` variable. The default is **Python 3.11**.

## Development

You can run the project locally with `docker compose`:

```raw
export S3_BUCKET=pypi-index-bucket
export MINIO_ROOT_USER=user
export MINIO_ROOT_PASSWORD=hunter2
```

```console
docker compose up --build -d
pip install --timeout=1000 -i http://localhost:4000 pandas --target ./package
```

Running the integration tests:

```console
pytest -s e2e/e2e.py
```
