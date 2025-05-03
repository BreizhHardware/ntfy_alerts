import requests
import time
import os
import logging
import sqlite3
import subprocess
import json
import threading

from send_ntfy import (
    github_send_to_ntfy,
    docker_send_to_ntfy,
)
from send_gotify import (
    github_send_to_gotify,
    docker_send_to_gotify,
)
from send_discord import (
    github_send_to_discord,
    docker_send_to_discord,
)

from send_slack import (
    github_send_to_slack,
    docker_send_to_slack,
)

# Configuring the logger
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)

github_token = os.environ.get("GHNTFY_TOKEN")
github_headers = {}
if github_token:
    github_headers["Authorization"] = f"token {github_token}"

docker_username = os.environ.get("DOCKER_USERNAME")
docker_password = os.environ.get("DOCKER_PASSWORD")

discord_webhook_url = os.environ.get("DISCORD_WEBHOOK_URL")


def create_dockerhub_token(username, password):
    url = "https://hub.docker.com/v2/users/login"
    headers = {"Content-Type": "application/json"}
    data = json.dumps({"username": username, "password": password})

    response = requests.post(url, headers=headers, data=data)

    if response.status_code == 200:
        token = response.json().get("token")
        if token:
            return token
        else:
            logger.error("Failed to get Docker Hub token.")
    else:
        logger.error(f"Failed to get Docker Hub token. Status code: {response.status_code}")
    return None


docker_token = create_dockerhub_token(docker_username, docker_password)
docker_header = {}
if docker_token:
    docker_header["Authorization"] = f"Bearer {docker_token}"
# Connecting to the database to store previous versions
conn = sqlite3.connect(
    "/github-ntfy/ghntfy_versions.db",
    check_same_thread=False,
)
cursor = conn.cursor()

# Creating the table if it does not exist
cursor.execute(
    """CREATE TABLE IF NOT EXISTS versions
                  (repo TEXT PRIMARY KEY, version TEXT, changelog TEXT)"""
)
conn.commit()

cursor.execute(
    """CREATE TABLE IF NOT EXISTS docker_versions
                  (repo TEXT PRIMARY KEY, digest TEXT)"""
)
conn.commit()

logger.info("Starting version monitoring...")

conn2 = sqlite3.connect("/github-ntfy/watched_repos.db", check_same_thread=False)
cursor2 = conn2.cursor()

cursor2.execute(
    """CREATE TABLE IF NOT EXISTS watched_repos
                    (id INTEGER PRIMARY KEY, repo TEXT)"""
)
conn2.commit()

cursor2.execute(
    """CREATE TABLE IF NOT EXISTS docker_watched_repos
                        (id INTEGER PRIMARY KEY, repo TEXT)"""
)
conn2.commit()


def get_watched_repos():
    cursor2.execute("SELECT * FROM watched_repos")
    watched_repos_rows = cursor2.fetchall()
    watched_repos = []
    for repo in watched_repos_rows:
        watched_repos.append(repo[1])
    return watched_repos


def get_docker_watched_repos():
    cursor2.execute("SELECT * FROM docker_watched_repos")
    watched_repos_rows = cursor2.fetchall()
    watched_repos = []
    for repo in watched_repos_rows:
        watched_repos.append(repo[1])
    return watched_repos


def start_api():
    subprocess.Popen(["python", "ntfy_api.py"])


def get_latest_releases(watched_repos):
    releases = []
    for repo in watched_repos:
        url = f"https://api.github.com/repos/{repo}/releases/latest"
        response = requests.get(url, headers=github_headers)
        if response.status_code == 200:
            release_info = response.json()
            changelog = get_changelog(repo)
            release_date = release_info.get("published_at", "Release date not available")
            releases.append(
                {
                    "repo": repo,
                    "name": release_info["name"],
                    "tag_name": release_info["tag_name"],
                    "html_url": release_info["html_url"],
                    "changelog": changelog,
                    "published_at": release_date,
                }
            )
        else:
            logger.error(f"Failed to fetch release info for {repo}")
    return releases


def get_latest_docker_releases(watched_repos):
    releases = []
    for repo in watched_repos:
        url = f"https://hub.docker.com/v2/repositories/{repo}/tags/latest"
        response = requests.get(url, headers=docker_header)
        if response.status_code == 200:
            release_info = response.json()
            release_date = release_info["last_upated"]
            digest = release_date["digest"]
            releases.append(
                {
                    "repo": repo,
                    "digest": digest,
                    "html_url": "https://hub.docker.com/r/" + repo,
                    "published_at": release_date,
                }
            )
        else:
            logger.error(f"Failed to fetch Docker Hub info for {repo}")
    return releases


def get_changelog(repo):
    url = f"https://api.github.com/repos/{repo}/releases"
    response = requests.get(url, headers=github_headers)
    if response.status_code == 200:
        releases = response.json()
        if releases:
            latest_release_list = releases[0]
            if "body" in latest_release_list:
                return latest_release_list["body"]
    return "Changelog not available"

def notify_all_services(github_latest_release, docker_latest_release, auth, ntfy_url, gotify_url, gotify_token, discord_webhook_url, slack_webhook_url):
    threads = []

    if ntfy_url:
        if github_latest_release:
            threads.append(threading.Thread(target=github_send_to_ntfy, args=(github_latest_release, auth, ntfy_url)))
        if docker_latest_release:
            threads.append(threading.Thread(target=docker_send_to_ntfy, args=(docker_latest_release, auth, ntfy_url)))

    if gotify_url and gotify_token:
        if github_latest_release:
            threads.append(threading.Thread(target=github_send_to_gotify, args=(github_latest_release, gotify_token, gotify_url)))
        if docker_latest_release:
            threads.append(threading.Thread(target=docker_send_to_gotify, args=(docker_latest_release, gotify_token, gotify_url)))

    if discord_webhook_url:
        if github_latest_release:
            threads.append(threading.Thread(target=github_send_to_discord, args=(github_latest_release, discord_webhook_url)))
        if docker_latest_release:
            threads.append(threading.Thread(target=docker_send_to_discord, args=(docker_latest_release, discord_webhook_url)))

    if slack_webhook_url:
        if github_latest_release:
            threads.append(threading.Thread(target=github_send_to_slack, args=(github_latest_release, slack_webhook_url)))
        if docker_latest_release:
            threads.append(threading.Thread(target=docker_send_to_slack, args=(docker_latest_release, slack_webhook_url)))
    
    for thread in threads:
        thread.start()

    for thread in threads:
        thread.join()



if __name__ == "__main__":
    start_api()
    with open("/auth.txt", "r") as f:
        auth = f.read().strip()
    ntfy_url = os.environ.get("NTFY_URL")
    gotify_url = os.environ.get("GOTIFY_URL")
    gotify_token = os.environ.get("GOTIFY_TOKEN")
    discord_webhook_url = os.environ.get("DISCORD_WEBHOOK_URL")
    timeout = float(os.environ.get("GHNTFY_TIMEOUT"))
    slack_webhook_url = os.environ.get("SLACK_WEBHOOK_URL")

    if auth and (ntfy_url or gotify_url or discord_webhook_url):
        while True:
            github_watched_repos_list = get_watched_repos()
            github_latest_release = get_latest_releases(github_watched_repos_list)
            docker_watched_repos_list = get_docker_watched_repos()
            docker_latest_release = get_latest_docker_releases(docker_watched_repos_list)

            notify_all_services(github_latest_release, docker_latest_release, auth, ntfy_url, gotify_url, gotify_token, discord_webhook_url, slack_webhook_url)

            time.sleep(timeout)
    else:
        logger.error("Usage: python ntfy.py")
        logger.error(
            "auth: can be generataed by the folowing command: echo -n 'username:password' | base64 and need to be "
            "stored in a file named auth.txt"
        )
        logger.error("NTFY_URL: the url of the ntfy server need to be stored in an environment variable named NTFY_URL")
        logger.error(
            "GOTIFY_URL: the url of the gotify server need to be stored in an environment variable named GOTIFY_URL"
        )
        logger.error(
            "GOTIFY_TOKEN: the token of the gotify server need to be stored in an environment variable named GOTIFY_TOKEN"
        )
        logger.error("DISCORD_WEBHOOK_URL: the webhook URL for Discord notifications need to be stored in an environment variable named DISCORD_WEBHOOK_URL")
        logger.error("GHNTFY_TIMEOUT: the time interval between each check")
