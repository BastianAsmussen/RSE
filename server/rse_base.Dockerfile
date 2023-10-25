FROM rust:latest

WORKDIR /usr/src/app
COPY ../database/lib ../database/lib

ENV RUST_LOG=INFO