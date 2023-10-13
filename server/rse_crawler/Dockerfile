FROM rust:1.73

WORKDIR /usr/bin/rse_crawler
COPY . .

RUN cargo install --path .

ENV RUST_LOG=INFO

CMD [ "rse_crawler" ]
