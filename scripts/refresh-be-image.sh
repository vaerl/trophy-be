docker stop eval-be
docker rm eval-be
docker rmi eval-be
docker build -t eval-be:latest ./be
docker run -p 5432:5432 --name eval-be -t eval-be
docker start eval-be
