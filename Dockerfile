FROM rust:latest as build
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV SQLX_OFFLINE=true

WORKDIR /usr/src/trophy-be

# install clang for libxlsxwriter
RUN apt-get update && apt-get install clang -y

# copy deriver_responder
RUN mkdir ../derive_responder
COPY ./derive_responder ../derive_responder

COPY ./be .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian10

COPY --from=build /usr/local/cargo/bin/trophy-be /usr/local/bin/trophy-be

CMD ["trophy-be"]
