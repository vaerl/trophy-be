-- drop existing tables and types
-- create enums
CREATE TYPE team_gender AS ENUM ('female', 'male', 'mixed');
CREATE TYPE game_type AS ENUM ('points', 'types');
-- create tables
CREATE TABLE game (
    game_id identity PRIMARY KEY,
    game_name varchar (50) NOT NULL,
    type game_type NOT NULL
);
CREATE TABLE team (
    team_id identity PRIMARY KEY,
    team_name varchar (50) NOT NULL,
    gender team_gender NOT NULL
);
CREATE TABLE game_team (
    game_id int REFERENCES game (game_id) ON UPDATE CASCADE ON DELETE CASCADE,
    team_id int REFERENCES team (team_id) ON UPDATE CASCADE,
    result text NOT NULL DEFAULT 1,
    CONSTRAINT game_team_pkey PRIMARY KEY (game_id, team_id) -- explicit pk
);