# ntfy_alerts
Personal ntfy alerts system


# Python ntfy.py
## Description:
This script is used to watch the github repos and send a notification to the ntfy server when a new release is published.
## Utilisation:
````python
python ntfy.py <auth> <ntfy_url>
````
auth: can be generataed by the folowing command: echo -n 'username:password' | base64
ntfy_url: the url of the ntfy server including the topic

Acctualy the watched repos list is hardcoded in the ntfy.py file under the name of watched_repos_list.
## TODO:
- [ ] Dockerize the ntfy.py
- [ ] Add the watched repos list as a parameter
- [ ] Add the watched repos list as a file
- [ ] Add the watched repos list as a database
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

