FROM python:3.11.8-alpine3.19

LABEL maintainer="BreizhHardware"

ADD ntfy.py /
ADD requirements.txt /
ADD entrypoint.sh /
RUN apk add --no-cache sqlite-dev sqlite-libs gcc musl-dev
RUN pip install -r requirements.txt

# Définir les variables d'environnement pour username et password
ENV USERNAME="" \
    PASSWORD="" \
    NTFY_URL="" \
    GHNTFY_TIMEOUT="3600" \
    GHREPO=""

ENTRYPOINT ["/entrypoint.sh"]
