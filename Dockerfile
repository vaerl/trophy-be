FROM rust:latest AS build
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV SQLX_OFFLINE=true

WORKDIR /usr/src/trophy-be

# install clang for libxlsxwriter
RUN apt-get update && apt-get install clang -y

# create the secret key
RUN head -c16 /dev/urandom > secret.key

COPY . .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian13:latest

COPY --from=build /usr/local/cargo/bin/trophy-be /usr/local/bin/trophy-be

CMD ["trophy-be"]
