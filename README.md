# Serverless PyPI Builder

An experimental PyPI dependency builder for Scaleway Serverless Python functions.

This project is aimed at simplifying the use of native libraries when using Python functions. Instead of building  your native dependencies on your laptop, you can deploy this project and build them on the public Cloud directly.

This project uses two containers on `min-scale=0` so you will only pay when running builds.

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

```console
export MY_INDEX=$(terraform output -raw index_url)

pip install --timeout=1000 -i $MY_INDEX psycopg2 --target ./package
```

## Zig toolchain

By default, the compilation of C libraries is done with [Zig](https://zig.news/kristoff/compile-a-c-c-project-with-zig-368j). From my own biased testing, the compilation is a bit faster with Zig than gcc.

## Known issues

### Python versions

Because of the way this works, it's important to have the same Python version between the worker and the runtime of your Python functions. This can be configured in Terraform via the `worker_python_version` variable. The default is **Python 3.11**.
