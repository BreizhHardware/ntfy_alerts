import requests
import sqlite3
import logging

conn = sqlite3.connect(
    "/github-ntfy/ghntfy_versions.db",
    check_same_thread=False,
)
cursor = conn.cursor()

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)


def github_send_to_ntfy(releases, auth, url):
    for release in releases:
        app_name = release["repo"].split("/")[-1]  # Getting the application name from the repo
        version_number = release["tag_name"]  # Getting the version number
        app_url = release["html_url"]  # Getting the application URL
        changelog = release["changelog"]  # Getting the changelog
        release_date = release["published_at"]  # Getting the release date
        release_date = release_date.replace("T", " ").replace("Z", "")  # Formatting the release date

        # Checking if the version has changed since the last time
        cursor.execute(
            "SELECT version FROM versions WHERE repo=?",
            (app_name,),
        )
        previous_version = cursor.fetchone()
        if previous_version and previous_version[0] == version_number:
            logger.info(f"The version of {app_name} has not changed. No notification sent.")
            continue  # Move on to the next application

        message = f"New version: {version_number}\nFor: {app_name}\nPublished on: {release_date}\nChangelog:\n{changelog}\n{app_url}"
        # Updating the previous version for this application
        cursor.execute(
            "INSERT OR REPLACE INTO versions (repo, version, changelog) VALUES (?, ?, ?)",
            (app_name, version_number, changelog),
        )
        conn.commit()

        headers = {
            "Authorization": f"Basic {auth}",
            "Title": f"New version for {app_name}",
            "Priority": "urgent",
            "Markdown": "yes",
            "Actions": f"view, Update {app_name}, {app_url}, clear=true",
        }
        response = requests.post(f"{url}", headers=headers, data=message)
        if response.status_code == 200:
            logger.info(f"Message sent to Ntfy for {app_name}")
            continue
        else:
            logger.error(f"Failed to send message to Ntfy. Status code: {response.status_code}")


def docker_send_to_ntfy(releases, auth, url):
    for release in releases:
        app_name = release["repo"].split("/")[-1]  # Getting the application name from the repo
        digest_number = release["digest"]
        app_url = release["html_url"]  # Getting the application URL
        release_date = release["published_at"]  # Getting the release date
        release_date = release_date.replace("T", " ").replace("Z", "")  # Formatting the release date

        # Checking if the version has changed since the last time
        cursor.execute(
            "SELECT digest FROM docker_versions WHERE repo=?",
            (app_name,),
        )
        previous_digest = cursor.fetchone()
        if previous_digest and previous_digest[0] == digest_number:
            logger.info(f"The digest of {app_name} has not changed. No notification sent.")
            continue  # Move on to the next application

        message = f"New version: {digest_number}\nFor: {app_name}\nPublished on: {release_date}\n{app_url}"
        # Updating the previous digest for this application
        cursor.execute(
            "INSERT OR REPLACE INTO docker_versions (repo, digest) VALUES (?, ?, ?)",
            (app_name, digest_number),
        )
        conn.commit()

        headers = {
            "Authorization": f"Basic {auth}",
            "Title": f"New version for {app_name}",
            "Priority": "urgent",
            "Markdown": "yes",
            "Actions": f"view, Update {app_name}, {app_url}, clear=true",
        }
        response = requests.post(f"{url}", headers=headers, data=message)
        if response.status_code == 200:
            logger.info(f"Message sent to Ntfy for {app_name}")
            continue
        else:
            logger.error(f"Failed to send message to Ntfy. Status code: {response.status_code}")
