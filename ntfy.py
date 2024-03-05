import requests
import time
import os
import logging
import sqlite3
import subprocess

# Configuring the logger
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

github_token = os.environ.get('GHNTFY_TOKEN')
github_headers = {}
if github_token:
    github_headers['Authorization'] = f"token {github_token}"

# Connecting to the database to store previous versions
conn = sqlite3.connect('/github-ntfy/ghntfy_versions.db', check_same_thread=False)
cursor = conn.cursor()

# Creating the table if it does not exist
cursor.execute('''CREATE TABLE IF NOT EXISTS versions
                  (repo TEXT PRIMARY KEY, version TEXT, changelog TEXT)''')
conn.commit()

logger.info("Starting version monitoring...")

conn2 = sqlite3.connect('/github-ntfy/watched_repos.db', check_same_thread=False)
cursor2 = conn2.cursor()

cursor2.execute('''CREATE TABLE IF NOT EXISTS watched_repos
                    (id INTEGER PRIMARY KEY, repo TEXT)''')
conn2.commit()


def get_watched_repos():
    cursor2.execute("SELECT * FROM watched_repos")
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
            release_date = release_info.get('published_at', 'Release date not available')
            releases.append({
                "repo": repo,
                "name": release_info["name"],
                "tag_name": release_info["tag_name"],
                "html_url": release_info["html_url"],
                "changelog": changelog,
                "published_at": release_date
            })
        else:
            logger.error(f"Failed to fetch release info for {repo}")
    return releases


def get_changelog(repo):
    url = f"https://api.github.com/repos/{repo}/releases"
    response = requests.get(url, headers=github_headers)
    if response.status_code == 200:
        releases = response.json()
        if releases:
            latest_release_list = releases[0]
            if 'body' in latest_release_list:
                return latest_release_list['body']
    return "Changelog not available"


def send_to_ntfy(releases, auth, url):
    for release in releases:
        app_name = release['repo'].split('/')[-1]  # Getting the application name from the repo
        version_number = release['tag_name']  # Getting the version number
        app_url = release['html_url']  # Getting the application URL
        changelog = release['changelog']  # Getting the changelog
        release_date = release['published_at'] # Getting the release date
        release_date = release_date.replace("T", " ").replace("Z", "") # Formatting the release date

        # Checking if the version has changed since the last time
        cursor.execute("SELECT version FROM versions WHERE repo=?", (app_name,))
        previous_version = cursor.fetchone()
        if previous_version and previous_version[0] == version_number:
            logger.info(f"The version of {app_name} has not changed. No notification sent.")
            continue  # Move on to the next application

        message = f"New version: {version_number}\nFor: {app_name}\nPublished on: {release_date}\nChangelog:\n{changelog}\n{app_url}"
        # Updating the previous version for this application
        cursor.execute("INSERT OR REPLACE INTO versions (repo, version, changelog) VALUES (?, ?, ?)",
                       (app_name, version_number, changelog))
        conn.commit()

        headers = {
            "Authorization": f"Basic {auth}",
            "Title": f"New version for {app_name}",
            "Priority": "urgent",
            "Markdown": "yes",
            "Actions": f"view, Update {app_name}, {app_url}, clear=true"}
        response = requests.post(f"{url}", headers=headers, data=message)
        if response.status_code == 200:
            logger.info(f"Message sent to Ntfy for {app_name}")
            continue
        else:
            logger.error(f"Failed to send message to Ntfy. Status code: {response.status_code}")


if __name__ == "__main__":
    start_api()
    with open('/auth.txt', 'r') as f:
        auth = f.read().strip()
    ntfy_url = os.environ.get('NTFY_URL')
    timeout = float(os.environ.get('GHNTFY_TIMEOUT'))

    if auth and ntfy_url:
        while True:
            watched_repos_list = get_watched_repos()
            latest_release = get_latest_releases(watched_repos_list)
            if latest_release:
                send_to_ntfy(latest_release, auth, ntfy_url)
            time.sleep(timeout)  # Wait an hour before checking again
    else:
        logger.error("Usage: python ntfy.py")
        logger.error(
            "auth: can be generataed by the folowing command: echo -n 'username:password' | base64 and need to be stored in a file named auth.txt")
        logger.error("NTFY_URL: the url of the ntfy server need to be stored in an environment variable named NTFY_URL")

