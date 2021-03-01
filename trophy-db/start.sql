-- drop existing enums
DROP TYPE IF EXISTS game_kind;
DROP TYPE IF EXISTS team_gender;
DROP TYPE IF EXISTS user_role;
-- drop existing tables
DROP TABLE IF EXISTS game;
DROP TABLE IF EXISTS team;
DROP TABLE IF EXISTS game_team;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS login_history;
DROP TABLE IF EXISTS transaction_history;
---
---
---
-- create enums
CREATE TYPE game_kind AS ENUM ('points', 'time');
CREATE TYPE team_gender AS ENUM ('female', 'male', 'mixed');
CREATE TYPE user_role AS ENUM ('admin', 'referee', 'visualizer');
-- create model-tables
CREATE TABLE users (
    id serial PRIMARY KEY NOT NULL,
    username varchar (50) NOT NULL,
    password varchar (50) NOT NULL,
    role user_role NOT NULL,
    session varchar NOT NULL DEFAULT ''
);
CREATE TABLE games (
    id serial PRIMARY KEY NOT NULL,
    name varchar (50) NOT NULL,
    kind game_kind NOT NULL,
    user_id int NOT NULL REFERENCES users (id) ON UPDATE CASCADE
);
CREATE TABLE teams (
    id serial PRIMARY KEY NOT NULL,
    name varchar (50) NOT NULL,
    gender team_gender NOT NULL,
    points integer NOT NULL DEFAULT 0
);
CREATE TABLE game_team (
    game_id int REFERENCES games (id) ON UPDATE CASCADE ON DELETE CASCADE,
    team_id int REFERENCES teams (id) ON UPDATE CASCADE,
    data text DEFAULT NULL,
    CONSTRAINT game_team_pkey PRIMARY KEY (game_id, team_id) -- explicit pk
);
--- create meta-tables
CREATE TABLE login_history (
    id serial PRIMARY KEY NOT NULL,
    user_id int NOT NULL REFERENCES users (id),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL
);
CREATE TABLE transaction_history (
    id serial PRIMARY KEY NOT NULL,
    user_id int NOT NULL REFERENCES users (id),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    action varchar NOT NULL
);