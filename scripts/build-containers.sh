# build the be-image
docker stop eval-be
docker rm eval-be
docker rmi eval-be
docker build -t eval-be:latest ./be
docker run -p 5432:5432 --name eval-be -t eval-be

# build the db-image
docker stop trophy-db
docker rm trophy-db
docker rmi trophy-db
docker build -t trophy-db:latest ./db
docker run -p 5432:5432 --name trophy-db -t trophy-db
docker start trophy-db
