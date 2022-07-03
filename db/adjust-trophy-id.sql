-- get all game-trophy-ids (after adding the column manually)
UPDATE game_team
SET game_trophy_id = games.trophy_id
FROM games
WHERE game_team.game_id = games.id;

-- do the same for team-trophy-ids
UPDATE game_team
SET team_trophy_id = teams.trophy_id
FROM teams
WHERE game_team.team_id = teams.id;
