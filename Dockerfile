FROM rust:1.67.1-alpine3.17 as build

RUN apk add --update --no-cache libc-dev

RUN USER=root cargo new --bin sls-pypi
WORKDIR /sls-pypi

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release \
    && rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/sls_pypi*
RUN cargo build --release

FROM alpine:3.17

WORKDIR /home/app

COPY --from=build /sls-pypi/target/release/sls-pypi .

ENV RUST_LOG=info

EXPOSE 4000
CMD ["/home/app/sls-pypi"]

