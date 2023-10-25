FROM rse_base

WORKDIR /usr/src/rse_server
COPY . .

RUN cargo build --release

CMD ["./target/release/rse_server"]