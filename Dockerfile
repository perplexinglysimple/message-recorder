FROM rust as builder

WORKDIR /usr/src/myapp

RUN apt update && apt install -y protobuf-compiler
COPY . .
RUN cargo test
RUN cargo install --path .

FROM debian:bookworm-slim

COPY --from=builder /usr/local/cargo/bin/message-recorder /usr/local/bin/message-recorder
COPY config config
CMD ["message-recorder"]