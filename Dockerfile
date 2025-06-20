FROM alpine:3.22

# Copier le binaire
COPY github-ntfy /usr/local/bin/github-ntfy

# Installer les dépendances
RUN apk add --no-cache sqlite-libs openssl nginx nodejs npm && \
    chmod +x /usr/local/bin/github-ntfy

WORKDIR /app

# Copier les fichiers web dans le répertoire attendu par nginx
COPY web /var/www/html/
COPY nginx.conf /etc/nginx/nginx.conf

# Copier le script d'entrée
COPY entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

# Créer le répertoire de données
RUN mkdir -p /github-ntfy && chmod 755 /github-ntfy

EXPOSE 5000 80 3000

ENTRYPOINT ["/app/entrypoint.sh"]
