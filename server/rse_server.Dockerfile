FROM rust:latest

WORKDIR /usr/src/app
COPY rse_server .
COPY database/lib ../database/lib

RUN cargo build --release

CMD ["./target/release/rse_server"]