import requests
import time
import os
import logging
import json

# Configurer le logger
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

repo_list_env = os.environ.get('GHREPO')
watched_repos_list = json.loads(repo_list_env) if repo_list_env else []

if not watched_repos_list:
    logger.error("Aucun dépôt n'a été spécifié. Veuillez spécifier les dépôts à surveiller dans l'environnement GHREPO")
    exit(1)

# Dictionnaire pour stocker les versions précédentes
previous_versions = {}

logger.info("Démarrage de la surveillance des versions...")

def get_latest_releases(watched_repos):
    releases = []
    for repo in watched_repos:
        url = f"https://api.github.com/repos/{repo}/releases/latest"
        response = requests.get(url)
        if response.status_code == 200:
            release_info = response.json()
            releases.append({
                "repo": repo,
                "name": release_info["name"],
                "tag_name": release_info["tag_name"],
                "html_url": release_info["html_url"]
            })
        else:
            logger.error(f"Failed to fetch release info for {repo}")
    return releases


def send_to_ntfy(releases, auth, url):
    for release in releases:
        app_name = release['repo'].split('/')[-1]  # Obtenir le nom de l'application à partir du repo
        version_number = release['tag_name']  # Obtenir le numéro de version
        app_url = release['html_url']  # Obtenir l'URL de l'application

        # Vérifier si la version a changé depuis la dernière fois
        if app_name in previous_versions and previous_versions[app_name] == version_number:
            logger.info(f"La version de {app_name} n'a pas changé. Pas de notification envoyée.")
            continue  # Passer à l'application suivante

        message = f"Nouvelle version: {version_number}\nPour: {app_name}\n{app_url}"
        # Mettre à jour la version précédente pour cette application
        previous_versions[app_name] = version_number

        headers = {"Authorization": f"Basic {auth}", "Content-Type": "text/plain"}
        response = requests.post(f"{url}", headers=headers, data=message)
        if response.status_code == 200:
            logger.info(f"Message envoyé à Ntfy pour {app_name}")
            continue
        else:
            logger.error(f"Échec de l'envoi du message à Ntfy. Code d'état : {response.status_code}")


if __name__ == "__main__":
    with open('/auth.txt', 'r') as f:
        auth = f.read().strip()
    ntfy_url = os.environ.get('NTFY_URL')
    timeout = float(os.environ.get('GHNTFY_TIMEOUT'))

    if auth and ntfy_url:
        while True:
            latest_release = get_latest_releases(watched_repos_list)
            if latest_release:
                send_to_ntfy(latest_release, auth, ntfy_url)
            time.sleep(timeout)  # Attendre une heure avant de vérifier à nouveau
    else:
        logger.error("Usage: python ntfy.py")
        logger.error("auth: can be generataed by the folowing command: echo -n 'username:password' | base64 and need to be stored in a file named auth.txt")
        logger.error("NTFY_URL: the url of the ntfy server need to be stored in an environment variable named NTFY_URL")
