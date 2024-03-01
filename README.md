# ntfy_alerts
Personal ntfy alerts system
feel free to contribute and to fork

# Python ntfy.py
## Description:
This script is used to watch the github repos and send a notification to the ntfy server when a new release is published.
## Utilisation:
auth and ntfy_url are required to be set as environment variables.

auth: can be generataed by the folowing command: echo -n 'username:password' | base64

ntfy_url: the url of the ntfy server including the topic

````python
python ntfy.py
````
## Docker:
If you want to use the docker image you can use the following docker-compose file:
````yaml
version: '3'
services:
  github-ntfy:
    image: breizhhardware/github-ntfy:latest
    container_name: github-ntfy
    environment:
      - USERNAME=username # Required
      - PASSWORD=password # Required
      - NTFY_URL=ntfy_url # Required
      - GHNTFY_TIMEOUT=timeout # Default is 3600 (1 hour)
      - GHREPO=["username/repo1", "username/repo2"] # Default is empty
      - GHNTFY_TOKEN= # Default is empty (Github token)
    restart: unless-stopped
````
GHNTFY_TOKEN, need to have repo, read:org and read:user
## TODO:
- [x] Dockerize the ntfy.py
- [x] Add the watched repos list as a parameter
- [x] Add the application version as a database
- [ ] Add the watched repos list as a web interface
# Bash setup-notify.sh
## Description:
This script is used to setup the ntfy notification system on ssh login for a new server.
## Utilisation:
````bash
bash setup-notify.sh <ntfy_url> <username> <password> <topic>
````
ntfy_url: the url of the ntfy server

username: the username of the user

password: the password of the user

topic: the topic of the notification

This script will create a send-notify.sh in the root of your disk and add the login-notify.sh to the /etc/profile.d/ folder.
# Bash send-notify.sh
## Description:
This script is used to send a notification to the ntfy server.
## Utilisation:
````bash
bash send-notify.sh <ntfy_url> <basic_auth> <topic> <message>
````
ntfy_url: the url of the ntfy server

basic_auth: the basic auth of the user

topic: the topic of the notification

message: the message of the notification

