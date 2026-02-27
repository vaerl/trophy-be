FROM rust:latest AS build-deps
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV SQLX_OFFLINE=true

WORKDIR /usr/src/trophy-be

# install clang for libxlsxwriter
RUN apt-get update && apt-get install clang -y

# create the secret key
RUN head -c16 /dev/urandom > secret.key

# ---- dependency caching ----
COPY Cargo.toml Cargo.lock ./

# create dummy code to make cargo resolve dependencies
RUN mkdir -p src && echo "fn main() { println!(\"dummy\"); }" > src/main.rs

# download and build dependencies
RUN cargo build --release || true

# ---- test ----
FROM build-deps AS test
WORKDIR /usr/src/trophy-be

# copy full source for tests
COPY . .

# run tests
RUN cargo test --workspace

# ---- build/install stage ----
FROM build-deps AS build
WORKDIR /usr/src/trophy-be

COPY . .

RUN cargo install --path .

# ---- final runtime image ----
FROM gcr.io/distroless/cc-debian13:latest

COPY --from=build /usr/local/cargo/bin/trophy-be /usr/local/bin/trophy-be

CMD ["trophy-be"]
