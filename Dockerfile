FROM rust as builder

WORKDIR /usr/src/myapp

RUN apt install protoc
COPY . .
RUN cargo test
RUN cargo install --path .

FROM debian:bookworm-slim

COPY --from=builder /usr/local/cargo/bin/message-recorder /usr/local/bin/message-recorder
CMD ["message-recorder"]