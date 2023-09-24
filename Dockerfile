FROM clux/muslrust:1.72.0-stable as builder

WORKDIR /usr/src/awair-exporter
COPY . .
RUN cargo build --release

FROM alpine:3.18
COPY --from=builder /usr/src/awair-exporter/target/x86_64-unknown-linux-musl/release/awair-exporter .

ENTRYPOINT ["/awair-exporter"]
