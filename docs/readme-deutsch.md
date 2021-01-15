# KL-Bak

// TODO (re)write this - after having finished the english version

## Aufbau

### Datenbank

Als Datenbank verwende ich [Postgres](https://www.postgresql.org/).

### Backend

Das Backend ist in [Rust](https://www.rust-lang.org/) geschrieben und verwendet [sqlx](https://github.com/launchbadge/sqlx) und [Actix-Web](https://github.com/actix/actix-web).

## Starten

Um Datenbank und Backend zu starten, muss [Docker](https://www.docker.com/) [Docker-Compose](https://docs.docker.com/compose/install/) installiert sein.
Dann muss lediglich `docker-compose up` im Terminal ausgeführt werden.
