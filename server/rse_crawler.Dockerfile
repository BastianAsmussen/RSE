FROM rust:latest

WORKDIR /usr/src/app
COPY rse_crawler .
COPY common ../common

RUN cargo build --release

ENV RUST_LOG=info
CMD ["./target/release/rse_crawler"]
