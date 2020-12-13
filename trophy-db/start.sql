-- drop existing tables and types
DROP TYPE IF EXISTS team_gender;
DROP TYPE IF EXISTS game_type;
DROP TABLE IF EXISTS game;
DROP TABLE IF EXISTS team;
DROP TABLE IF EXISTS game_team;
-- create enums
CREATE TYPE gender AS ENUM ('female', 'male', 'mixed');
CREATE TYPE type AS ENUM ('points', 'types');
-- create tables
CREATE TABLE game (
    id serial PRIMARY KEY,
    name varchar (50) NOT NULL,
    type type NOT NULL
);
CREATE TABLE team (
    id serial PRIMARY KEY,
    name varchar (50) NOT NULL,
    gender gender NOT NULL
);
CREATE TABLE game_team (
    game_id int REFERENCES game (id) ON UPDATE CASCADE ON DELETE CASCADE,
    team_id int REFERENCES team (id) ON UPDATE CASCADE,
    result text NOT NULL DEFAULT 1,
    CONSTRAINT game_team_pkey PRIMARY KEY (game_id, team_id) -- explicit pk
);