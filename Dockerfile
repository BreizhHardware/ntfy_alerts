FROM python:3.11.8-alpine3.19

LABEL maintainer="BreizhHardware"

ADD ntfy.py /
ADD ntfy_api.py /
ADD migration.py /migration.py /
ADD requirements.txt /
ADD entrypoint.sh /
ADD index.html /var/www/html/index.html /
ADD script.js /var/www/html/script.js /
RUN apk add --no-cache sqlite-dev sqlite-libs gcc musl-dev nginx
RUN pip install -r requirements.txt

# Définir les variables d'environnement pour username et password
ENV USERNAME="" \
    PASSWORD="" \
    NTFY_URL="" \
    GHNTFY_TIMEOUT="3600" \
    GHNTFY_TOKEN="" \
    FLASK_ENV=production

# Exposer le port 5000 pour l'API et le port 80 pour le serveur web
EXPOSE 5000 80

COPY nginx.conf /etc/nginx/nginx.conf

ENTRYPOINT ["/entrypoint.sh"]
