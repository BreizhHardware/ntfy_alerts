FROM python:3.11.8-alpine3.19

LABEL maintainer="BreizhHardware"
LABEL version_number="1.4"

ADD ntfy.py /
ADD ntfy_api.py /
ADD requirements.txt /
ADD entrypoint.sh /
ADD send_ntfy.py /
ADD send_gotify.py /
ADD send_discord.py /
ADD index.html /var/www/html/index.html
ADD script.js /var/www/html/script.js
RUN apk add --no-cache sqlite-dev sqlite-libs musl-dev nginx gcc
RUN pip install -r requirements.txt
RUN chmod 700 /entrypoint.sh

# Définir les variables d'environnement pour username et password
ENV USERNAME="" \
    PASSWORD="" \
    NTFY_URL="" \
    GHNTFY_TIMEOUT="3600" \
    GHNTFY_TOKEN="" \
    DOCKER_USERNAME="" \
    DOCKER_PASSWORD="" \
    GOTIFY_URL="" \
    GOTIFY_TOKEN="" \
    DISCORD_WEBHOOK_URL="" \
    SLACK_WEBHOOK_URL="" \
    FLASK_ENV=production

# Exposer le port 5000 pour l'API et le port 80 pour le serveur web
EXPOSE 5000 80

COPY nginx.conf /etc/nginx/nginx.conf

ENTRYPOINT ["/entrypoint.sh"]
