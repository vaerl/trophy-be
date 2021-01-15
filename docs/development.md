# Development of KL-Bak

## Model

- sqlx maps the SERIAL-Postgres-Type to a i32, thus I'm not able to switch to an u32
  - I could consider using the [Newtype-Pattern](https://doc.rust-lang.org/rust-by-example/generics/new_types.html) for ID, but that would encompass an light extra-layer of mapping

## Documentation

- currently, there is no Swagger-Doc-Support(or similar) readily available -> CHECK in some months
