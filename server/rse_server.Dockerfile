FROM rust:latest

WORKDIR /usr/src/app
COPY rse_server .
COPY database/lib ../database/lib

RUN cargo build --release

ENV RUST_LOG=rse_server
CMD ["./target/release/rse_server"]