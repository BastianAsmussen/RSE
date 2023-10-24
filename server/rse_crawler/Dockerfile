ARG BINARY_NAME=rse_crawler

FROM rust:latest
ARG BINARY_NAME

WORKDIR /usr/src/${BINARY_NAME}
COPY . .

VOLUME ["/usr/src/database/lib"]

RUN cargo install --path .

ENV RUST_LOG=INFO
CMD ["rse_crawler"]