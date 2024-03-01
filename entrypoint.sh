#!/bin/sh

# Génère le contenu du fichier auth.txt à partir des variables d'environnement
echo -n "$USERNAME:$PASSWORD" | base64 > /auth.txt

# Exécute le script Python
exec python ./ntfy.py
