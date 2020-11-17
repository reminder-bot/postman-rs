# Postman

## Software for sending web requests from MySQL

### .env structure:
* `DISCORD_TOKEN`: The token used to send reminders to DMs or without webhooks
* `DATABASE_URL`: The path to the database you're using (MySQL)
* `INTERVAL`: How often to query database for new reminders

### Commandline Arguments
* `--dry-run`: if set, disables modifications to the database and the actual sending of reminders
