ARG PYTHON_VERSION=3.11
FROM "rg.fr-par.scw.cloud/scwfunctionsruntimes-public/python-dep":${PYTHON_VERSION}

RUN apk add \
    --no-cache -X http://dl-cdn.alpinelinux.org/alpine/edge/testing \
    --update --no-cache zig

WORKDIR /code

COPY ./requirements.txt /code/requirements.txt

RUN pip install --no-cache-dir --upgrade -r /code/requirements.txt

COPY ./worker.py /code/

CMD ["uvicorn", "worker:app", "--proxy-headers", "--host", "0.0.0.0", "--port", "8080"]