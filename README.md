# eval-be

- [Deutsche Version](./docs/readme-deutsch.md)
- [Development-Information](./docs/development.md)

## Evaluation

This software evaluates almost exactly like the previous, excel-based mechanism:
First, all outcomes of a game are fetched. Then the outcomes are separated by gender and sorted by better time or more points. Finally, points are assigned based on the resulting order, starting at 50. This backend handles the same score differently:

Example:

A -> 100 points
B -> 100 points
C -> 90 points

While A and B get 50 points and C 48 in the excel-version, c now gets 49 points.

## Running the backend

To run this project, install [Docker](https://docker.com) and Docker Compose(which should come bundled on Linux).
Then, do the following:

1. clone the project with Git
2. create the containers
3. create the environment
   1. create the secret-key
   2. create a .env-file
4. run `docker-compose up`

### Creating the containers

- [setup on caprover](docs/caprover-setup.md)

### Creating a secret key

Run `head -c16 /dev/urandom > secret.key`. TODO check if this is secure!

### Creating the .env-file

See [this](./be/.env-example) for an example. Just copy this file into a new `.env`-file and set the values accordingly.

## Websocket

To create a new lobby, simply connect to `ws/{id}` and supply a Uuid.
For example: `ws://localhost:5000/ws/0000002a-000c-0005-0c03-0938362b0809`
