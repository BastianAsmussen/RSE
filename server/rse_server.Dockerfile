FROM rust:latest

WORKDIR /usr/src/app
COPY rse_server .
COPY common/errors ../common/errors
COPY common/database ../common/database
COPY common/utils ../common/utils

RUN cargo build --release

ENV RUST_LOG=rse_server
CMD ["./target/release/rse_server"]
