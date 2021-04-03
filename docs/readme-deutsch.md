# eval-be

## Auswertung

Die Auswertung funktioniert beinahe exakt so wie bei der Excel-Tabelle:
Zuerst werden alle Ergebnisse des Spiels geladen und nach Geschlecht getrennt. Anschließend werden die Ergebnisse nach Zeit(aufsteigend) oder Punkten(absteigend) sortiert. Nun werden auf Basis der Reihenfolge beginnend mit 50 Punkte zugewiesen. Bei einem Gleichstand wird die selbe Punktzahl verwendet. Anders als bei der Excel-Tabelle wird mit der nächsten Zahl weitergerechnet.

Beispiel:

A -> 100 Punkte
B -> 100 Punkte
C -> 90 Punkte

In der Excel-Version bekommen A und B 50 Punkte und C 48. Hier bekommt C allerdings 49 Punkte.

## Backend starten

Um das Backend zu starten, muss [Docker](https://docker.com) Docker Compose installiert sein.
Anschließend müssen folgende Schritte erledigt werden:

1. Projekt mit Git clonen
2. Container erstellen
3. Umgebung erstellen
   1. Secret-Key erstellen
   2. Umgebungs-Datei erstellen
4. Starten mit `docker-compose up`

### Container erstellen

Für das Erstellen existieren Skripte in `/scripts`.
Um die Container zu erstellen, führe `build-containers.sh` aus.

### Secret-Key erstellen

Führe `head -c16 /dev/urandom > secret.key` aus.

### Umgebungs-Datei erstellen

[Hier](./be/.env-example) ist ein Beispiel. Die Datei muss nach `.env` kopiert und entsprechend ausgefüllt werden.
