import requests
import sqlite3
import logging

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)

def get_db_connection():
    return sqlite3.connect("/github-ntfy/ghntfy_versions.db", check_same_thread=False)

def github_send_to_discord(releases, webhook_url):
    conn = get_db_connection()
    cursor = conn.cursor()
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


        embeds = {
                "title": f"New version for {app_name}",
                "url": app_url,
                "color": "#027d21",
                "description": f"New version: {version_number}\nPublished on: {release_date}\nChangelog:\n{changelog}"
        }

        data = {
            "content": "New version available",
            "username": "GitHub Ntfy",
            "embeds": [embeds]
        }
        headers = {
            "Content-Type": "application/json"
        }
        response = requests.post(webhook_url, json = data, headers = headers)
        if 200 <= response.status_code < 300:
            logger.info(f"Message sent to Discord for {app_name}")
        else:
            logger.error(f"Failed to send message to Discord. Status code: {response.status_code}")

def docker_send_to_discord(releases, webhook_url):
    conn = get_db_connection()
    cursor = conn.cursor()
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

        # Creating the embed message
        embed = {
            "title": f"New version for {app_name}",
            "url": app_url,
            "color": "#027d21",
            "fields": [
                {
                    "name": "Digest",
                    "value": digest_number,
                    "inline": True
                },
                {
                    "name": "Published on",
                    "value": release_date,
                    "inline": True
                }
            ]
        }

        data = {
            "embeds": [embed]
        }
        response = requests.post(webhook_url, json=data)
        if response.status_code == 204:
            logger.info(f"Message sent to Discord for {app_name}")
        else:
            logger.error(f"Failed to send message to Discord. Status code: {response.status_code}")