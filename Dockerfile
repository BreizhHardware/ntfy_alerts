FROM python:3.11.8-alpine3.19

LABEL maintainer="BreizhHardware"

ADD ntfy.py /
ADD requirements.txt /
RUN pip install -r requirements.txt

# Définir les variables d'environnement pour username et password
ENV USERNAME="" \
    PASSWORD="" \
    NTFY_URL="" \
    GHNTFY_TIMEOUT="3600"

# Exécuter la commande pour générer l'authentification base64 à partir des variables d'environnement
RUN echo -n "$USERNAME:$PASSWORD" | base64 > /auth.txt

CMD ["python", "./ntfy.py"]
