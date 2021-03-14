FROM postgres:latest
ENV POSTGRES_USER postgres
ENV POSTGRES_PASSWORD admin
ENV POSTGRES_DB trophy
ADD start.sql /docker-entrypoint-initdb.d/
