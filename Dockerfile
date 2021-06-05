# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:latest as cargo-build

ENV SQLX_OFFLINE=true

RUN apt-get update

RUN apt-get install musl-tools clang -y

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/trophy-be

COPY ./be/Cargo.toml Cargo.toml

RUN mkdir ../derive_responder
COPY ./derive_responder ../derive_responder

RUN mkdir src/

RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

RUN rm -f target/x86_64-unknown-linux-musl/release/deps/trophy-be*

COPY ./be .

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

RUN addgroup -g 1000 trophy-be

RUN adduser -D -s /bin/sh -u 1000 -G trophy-be trophy-be

WORKDIR /home/trophy-be/bin/

COPY --from=cargo-build /usr/src/trophy-be/target/x86_64-unknown-linux-musl/release/trophy-be .

RUN chown trophy-be:trophy-be trophy-be

USER trophy-be

CMD ["./trophy-be"]

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl