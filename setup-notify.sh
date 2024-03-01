#!/bin/bash

# Script pour créer les fichiers login-notify.sh et send-notify.sh dans /etc/profile.d/

# Contenu du fichier login-notify.sh
LOGIN_NOTIFY_CONTENT="#!/bin/bash

NTFY_URL=\$1
username=\$2
password=\$3
Basic=\$(echo -n \"\$username:\$password\" | base64)
TOPIC=\$4

IP=\"\$(echo \"\${SSH_CONNECTION}\" | awk -F' ' '{print \$1}')\"  # ne fonctionne pas pour une connexion locale
DATE=\"\$(date)\"
NAME=\"\$(whoami)\"
NB_USERS=\$(who | wc -l)

MESSAGE=\"
New login to \${HOSTNAME} server!
\\\"\${NAME}\\\" from \\\"\${IP}\\\" 
\${NB_USERS} users connected
\${DATE}
\"

/send-notify.sh \"\${NTFY_URL}\" \"\${Basic}\" \"\${TOPIC}\" \"\${MESSAGE}\"
"

# Contenu du fichier send-notify.sh
SEND_NOTIFY_CONTENT="#!/bin/bash

set -eu

if [ \"\$#\" -ne 4 ]; then
  echo \"Usage: \$0 <ntfy_url> <basic_auth> <topic> <text message>\"
  echo \"Help basic_auth: \\\"echo -n 'testuser:fakepassword' | base64\\\"\"
  exit 1
fi

NTFY_URL=\$1
BASIC_AUTH=\$2
TOPIC=\$3
TEXT=\$4

RES=\$(curl -i -s -X POST -H \"Authorization: Basic \${BASIC_AUTH}\" -d \"\${TEXT}\" \"\${NTFY_URL}/\${TOPIC}\")
STATUS_CODE=\$(echo \"\$RES\" | head -n 1 | awk -F' ' '{print \$2}')

if [[ \$STATUS_CODE -ne 200 ]] ; then
  echo \"error while sending alert\"
  echo \"\${RES}\"
  exit 1
fi
"

# Création des fichiers
echo "$LOGIN_NOTIFY_CONTENT" | sudo tee /etc/profile.d/login-notify.sh > /dev/null
echo "$SEND_NOTIFY_CONTENT" | sudo tee /send-notify.sh > /dev/null

# Attribution des permissions
sudo chmod +x /etc/profile.d/login-notify.sh /send-notify.sh

echo "Fichiers créés avec succès dans /etc/profile.d/ et /"

