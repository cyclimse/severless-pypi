ARG PYTHON_VERSION=3.11
FROM "rg.fr-par.scw.cloud/scwfunctionsruntimes-public/python-dep":${PYTHON_VERSION}

RUN apk add \
    --no-cache -X http://dl-cdn.alpinelinux.org/alpine/edge/testing \
    --update --no-cache zig binutils

WORKDIR /code

COPY ./requirements.txt /code/requirements.txt

RUN pip install --upgrade pip && \
    pip install --no-cache-dir -r /code/requirements.txt

COPY ./worker.py /code/

CMD ["uvicorn", "worker:app", "--proxy-headers", "--timeout-keep-alive", "900", "--host", "0.0.0.0", "--port", "8080"]