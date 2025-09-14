# eval-be

Backend for evaluating the [Klostertrophy](https://klostertrophy.de/).

- [Development-Information](./docs/development.md)

## Evaluation

First, all outcomes of a game are fetched.
Then, outcomes are separated by gender and sorted by time(ascending, since shorter is better) or points(descending, since more is better).
Finally, points are assigned based on the resulting order, starting at 50.

Here are some examples:

```text
A -> 100 seconds
B -> 100 seconds
C -> 90 seconds

A and B get 50 points, C gets 48.
```

```text
A -> 100 seconds
B -> 90 seconds
C -> 80 seconds

A gets 50, B get 49 and C gets 48 points.
```

```text
A -> 100 seconds
B -> 90 seconds
C -> 90 seconds

A gets 50, B and C get 49 points.
```

## Running the backend

1. create the dotenv-file, see [the example](example.env) for values that need to be set
2. create a secret: `head -c16 /dev/urandom > secret.key`
