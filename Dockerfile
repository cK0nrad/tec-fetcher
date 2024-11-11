FROM rust:1.82.0-slim

ENV SQLX_OFFLINE true
RUN apt-get install -y wget
RUN apt-get update && apt-get upgrade -y && apt-get install -y openssl libssl-dev pkg-config protobuf-compiler

WORKDIR /usr/src/app
COPY Cargo.toml .
COPY Cargo.lock .
COPY build.rs .

COPY src src
RUN cargo install --path .

EXPOSE ${PORT}
CMD ["tec-fetcher"]