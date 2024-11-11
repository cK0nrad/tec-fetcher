FROM rust:1.82.0-alpine3.20

# Path: /usr/src/app
WORKDIR /usr/src/app

# Path: /usr/src/app/Cargo.toml
COPY Cargo.toml .

# Path: /usr/src/app/Cargo.lock
COPY Cargo.lock .

COPY build.rs .

# Path: /usr/src/app/src
COPY src src

ENV SQLX_OFFLINE true

RUN apt-get update && apt-get upgrade -y && apt-get install -y openssl libssl-dev pkg-config protobuf-compiler

# Path: /usr/src/app
RUN cargo install --path .

EXPOSE ${PORT}

# for healthcheck
RUN apt-get install -y wget


CMD ["tec-fetcher"]