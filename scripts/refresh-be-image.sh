docker stop trophy-be
docker rm trophy-be
docker rmi trophy-be
docker build -t trophy-be:latest ./be
docker run -p 8080:8080 --name trophy-be -t trophy-be
docker start trophy-be
