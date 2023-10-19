ARG BINARY_NAME=rse_server

FROM rust:latest
ARG BINARY_NAME

WORKDIR /usr/src/${BINARY_NAME}
COPY . .

RUN cargo install --path .

ENV RUST_LOG=INFO
CMD ["rse_server"]

EXPOSE 8080