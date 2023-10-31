FROM rust:latest

WORKDIR /usr/src/app
COPY rse_crawler .
COPY error/lib ../error/lib
COPY database/lib ../database/lib

RUN cargo build --release

ENV RUST_LOG=rse_crawler
CMD ["./target/release/rse_crawler"]