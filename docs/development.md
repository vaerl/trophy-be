# Development of KL-Bak

## Model

- sqlx maps the SERIAL-Postgres-Type to a i32, thus I'm not able to switch to an u32
  - I could consider using the [Newtype-Pattern](https://doc.rust-lang.org/rust-by-example/generics/new_types.html) for ID, but that would encompass an light extra-layer of mapping

## Documentation

There is no Swagger-Doc-Support(or similar) readily available for rust.

## Error-Handling

Errors are propagated using the `anyhow`-crate. This allows to automatically cast to custom errors which implement `actix_web`'s `error::ResponseError`. This in turn allows just returning a result of type `Result<T, impl ResponseError>` for simple types or calling `err.error_response()` in a `match` for complex types.
