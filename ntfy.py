import requests
import time
from sys import argv as args

watched_repos_list = ['dani-garcia/vaultwarden', 'jellyfin/jellyfin', 'linuxserver/Heimdall',
                      'jlesage/docker-nginx-proxy-manager', 'linuxserver/docker-speedtest-tracker',
                      'linuxserver/docker-xbackbone', 'Fallenbagel/jellyseerr', 'FlareSolverr/FlareSolverr',
                      'linuxserver/docker-jackett', 'linuxserver/docker-radarr', 'linuxserver/docker-sonarr']

# Dictionnaire pour stocker les versions précédentes
previous_versions = {}


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
            print(f"Failed to fetch release info for {repo}")
    return releases


def send_to_ntfy(releases, auth, url):
    for release in releases:
        app_name = release['repo'].split('/')[-1]  # Obtenir le nom de l'application à partir du repo
        version_number = release['tag_name']  # Obtenir le numéro de version
        app_url = release['html_url']  # Obtenir l'URL de l'application

        # Vérifier si la version a changé depuis la dernière fois
        if app_name in previous_versions and previous_versions[app_name] == version_number:
            print(f"La version de {app_name} n'a pas changé. Pas de notification envoyée.")
            continue  # Passer à l'application suivante

        message = f"Nouvelle version: {version_number}\nPour: {app_name}\n{app_url}"
        # Mettre à jour la version précédente pour cette application
        previous_versions[app_name] = version_number

        headers = {"Authorization": f"Basic {auth}", "Content-Type": "text/plain"}
        response = requests.post(f"{url}", headers=headers, data=message)
        if response.status_code == 200:
            continue
        else:
            print(f"Échec de l'envoi du message à Ntfy. Code d'état : {response.status_code}")


if __name__ == "__main__":
    if len(args) == 3:
        while True:
            latest_release = get_latest_releases(watched_repos_list)
            if latest_release:
                send_to_ntfy(latest_release, args[1], args[2])
            time.sleep(3600)  # Attendre une heure avant de vérifier à nouveau
    else:
        print("Usage: python ntfy.py <auth> <ntfy_url>")
        print("auth: can be generataed by the folowing command: echo -n 'username:password' | base64")
