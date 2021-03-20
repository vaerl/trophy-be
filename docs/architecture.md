# Architecture

## Result-Type

I've introduced a type-alias for my results: `type ApiResult<T> = Result<T, CustomError>`.
Using this in conjunction with `CustomError` saves a lot of redundant casting.

## Actix Web

### NewType for Vecs

Actix refuses to return `Vec`s of types that implement `Responder`. Due to that, I've wrapped the `Vec`s in a new type which is typically called `<Type>Vec`. While this seems redundant I find it better than matching and manually returning an `HttpResponse`.
