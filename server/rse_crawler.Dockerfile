FROM rust:latest

WORKDIR /usr/src/app
COPY rse_crawler .
COPY common/errors ../common/errors
COPY common/database ../common/database
COPY common/utils ../common/utils

RUN cargo build --release

ENV RUST_LOG=info
CMD ["./target/release/rse_crawler"]
