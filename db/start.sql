-- drop existing tables
DROP TABLE IF EXISTS game_team;
DROP TABLE IF EXISTS transaction_history;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS games;
DROP TABLE IF EXISTS teams;
-- drop existing enums
DROP TYPE IF EXISTS game_kind;
DROP TYPE IF EXISTS team_gender;
DROP TYPE IF EXISTS user_role;
DROP TYPE IF EXISTS log_level;
---
---
---
-- create enums
CREATE TYPE game_kind AS ENUM ('points', 'time');
CREATE TYPE team_gender AS ENUM ('female', 'male', 'mixed');
CREATE TYPE user_role AS ENUM ('admin', 'referee', 'visualizer');
CREATE TYPE log_level AS ENUM ('debug', 'info', 'important');
-- create model-tables
CREATE TABLE games (
    id serial PRIMARY KEY NOT NULL,
    trophy_id integer NOT NULL,
    name varchar (50) NOT NULL,
    kind game_kind NOT NULL,
    locked boolean DEFAULT FALSE NOT NULL
);
CREATE TABLE users (
    id serial PRIMARY KEY NOT NULL,
    username varchar (50) NOT NULL,
    password varchar NOT NULL,
    role user_role NOT NULL,
    session varchar NOT NULL DEFAULT '',
    game_id int REFERENCES games (id)
);
CREATE TABLE teams (
    id serial PRIMARY KEY NOT NULL,
    trophy_id integer NOT NULL,
    name varchar (50) NOT NULL,
    gender team_gender NOT NULL,
    points integer NOT NULL DEFAULT 0
);
CREATE TABLE game_team (
    game_id int REFERENCES games (id) ON UPDATE CASCADE ON DELETE CASCADE,
    team_id int REFERENCES teams (id) ON UPDATE CASCADE ON DELETE CASCADE,
    data text DEFAULT NULL,
    CONSTRAINT game_team_pkey PRIMARY KEY (game_id, team_id) -- explicit pk
);
--- create meta-tables
CREATE TABLE transaction_history (
    id serial PRIMARY KEY NOT NULL,
    user_id int NOT NULL REFERENCES users (id),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    log_level log_level NOT NULL,
    action varchar NOT NULL
);
