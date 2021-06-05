FROM rust:latest as build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/trophy-be
COPY . .
RUN ls -la

RUN cargo install --path ./be

FROM gcr.io/distroless/cc-debian10

COPY --from=build /usr/local/cargo/bin/trophy-be /usr/local/bin/trophy-be

CMD ["trophy-be"]
