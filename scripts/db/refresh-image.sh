docker stop trophy-db
docker rm trophy-db
docker rmi trophy-db
docker build -t trophy-db:latest ./db
docker run -p 5432:5432 --name trophy-db -t trophy-db
docker start trophy-db