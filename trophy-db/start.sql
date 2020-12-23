-- drop existing tables and types
DROP TYPE IF EXISTS game_kind;
DROP TYPE IF EXISTS team_gender;
DROP TABLE IF EXISTS game;
DROP TABLE IF EXISTS team;
DROP TABLE IF EXISTS game_team;
-- create enums
CREATE TYPE game_kind AS ENUM ('points', 'time');
CREATE TYPE team_gender AS ENUM ('female', 'male', 'mixed');
-- create tables
CREATE TABLE game (
    id serial PRIMARY KEY,
    name varchar (50) NOT NULL,
    kind game_kind NOT NULL
);
CREATE TABLE team (
    id serial PRIMARY KEY,
    name varchar (50) NOT NULL,
    gender team_gender NOT NULL,
    points integer NOT NULL DEFAULT 0
);
CREATE TABLE game_team (
    game_id int REFERENCES game (id) ON UPDATE CASCADE ON DELETE CASCADE,
    team_id int REFERENCES team (id) ON UPDATE CASCADE,
    -- TODO is this ok?
    data text DEFAULT NULL,
    CONSTRAINT game_team_pkey PRIMARY KEY (game_id, team_id) -- explicit pk
);