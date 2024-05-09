# build the image
docker build -t trophy-be .

# build the image without any caching - caching caused some confusion early on
docker build -t trophy-be . --no-cache

# run the image with the applicable environment
docker run --name trophy-be -p 4998:4998 -d --env-file ./.env --network=base trophy-be

# run bash inside the container
docker exec -it trophy-be bash

# connect the container to the network, typically named base
docker network connect base trophy-be