FROM rust:1.82.0-slim

ENV SQLX_OFFLINE true
RUN apt-get update && apt-get upgrade -y && apt-get install -y openssl libssl-dev pkg-config protobuf-compiler wget

RUN cargo install sqlx-cli --no-default-features --features postgres

WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

EXPOSE ${PORT}
CMD ["/usr/src/app/docker_startup.sh"]