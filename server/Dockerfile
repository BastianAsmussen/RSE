FROM rust:1.73-buster as builder

WORKDIR /usr/bin/server
COPY . .

RUN cargo install --path .

# Possibly implement a runner?
# FROM debian:bullseye-slim
# 
# RUN apt-get update && \
#     rm -rf /var/lib/apt/lists/*
# 
# COPY --from=builder /usr/local/cargo/bin/server /usr/local/bin/server

CMD ["server"]
