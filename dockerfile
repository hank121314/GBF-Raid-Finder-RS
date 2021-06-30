FROM rust:1.53-alpine3.13 AS builder
WORKDIR /usr/src/
ENV RUSTFLAGS=-Ctarget-feature=-crt-static 
RUN apk update && apk add --no-cache openssl-dev \
  musl-dev \
  libgcc \
  llvm-libunwind \
  pkgconfig

RUN cargo new raid-finder
WORKDIR /usr/src/raid-finder
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM alpine:3.13.0
RUN apk update && apk add --no-cache openssl-dev \
  llvm-libunwind \
  libcurl=7.64.1 \
  libgcc
EXPOSE 50051
COPY --from=builder /usr/src/raid-finder/target/release/raid-finder raid-finder
CMD ["./raid-finder"]