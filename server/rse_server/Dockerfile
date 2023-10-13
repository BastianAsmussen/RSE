FROM rust:1.73

WORKDIR /usr/bin/rse_server
COPY . .

RUN cargo install --path .

ENV RUST_LOG=INFO

CMD [ "rse_server" ]
