#!/bin/sh

# Génère le contenu du fichier auth.txt à partir des variables d'environnement
echo -n "$USERNAME:$PASSWORD" | base64 > /auth.txt

# Démarrer nginx en arrière-plan
nginx -g 'daemon off;' &


# Démarrer l'API principale
exec /usr/local/bin/github-ntfy