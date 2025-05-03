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

def github_send_to_slack(releases, webhook_url):
    conn = get_db_connection()
    cursor = conn.cursor()
    for release in releases: 
        app_name = release["repo"].split("/")[-1]
        version_number = release["tag_name"]
        app_url = release["html_url"]
        changelog = release["changelog"]
        release_date = release["published_at"].replace("T", " ").replace("Z", "")

        cursor.execute("SELECT version FROM versions WHERE repo=?", (app_name,))
        previous_version = cursor.fetchone()
        if previous_version and previous_version[0] == version_number:
            logger.info(f"The version of {app_name} has not changed. No notification sent.")
            continue

        message = f"📌 *New version*: {version_number}\n\n📦*For*: {app_name}\n\n📅 *Published on*: {release_date}\n\n📝 *Changelog*:\n\n```{changelog}```"
        if len(message) > 2000:
             message = f"📌 *New version*: {version_number}\n\n📦*For*: {app_name}\n\n📅 *Published on*: {release_date}\n\n📝 *Changelog*:\n\n `check url since the changelog is huge`"

        cursor.execute(
            "INSERT OR REPLACE INTO versions (repo, version, changelog) VALUES (?, ?, ?)",
            (app_name, version_number, changelog),
        )
        conn.commit()


        message = {
            "blocks": [
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": f"{message}"
                    },
                    "accessory": {
                        "type": "button",
                        "text": {
                            "type": "plain_text",
                            "text": "🔗 Changelog url"
                        },
                        "url": f"{app_url}",
                        "action_id": "button-action"
                    }
                },
                {
                    "type": "divider"
                }
            ]
        }
        headers = {
            "Content-Type": "application/json"
        }
        response = requests.post(webhook_url, json=message, headers=headers)
        if response.status_code == 200:
            logger.info(f"Message sent to Slack for {app_name}")
        else:
            logger.error(f"Failed to send message to Slack. Status code: {response.status_code}")
            logger.error(f"Response: {response.text}")
    conn.close()

def docker_send_to_slack(releases, webhook_url):
    conn = get_db_connection()
    cursor = conn.cursor()
    for release in releases:
        app_name = release["repo"].split("/")[-1]
        digest_number = release["digest"]
        app_url = release["html_url"]
        release_date = release["published_at"].replace("T", " ").replace("Z", "")

        cursor.execute("SELECT digest FROM docker_versions WHERE repo=?", (app_name,))
        previous_digest = cursor.fetchone()
        if previous_digest and previous_digest[0] == digest_number:
            logger.info(f"The digest of {app_name} has not changed. No notification sent.")
            continue

        message = f"🐳 *Docker Image Updated!*\n\n🔐 *New Digest*: `{digest_number}`\n\n📦 *App*: {app_name}\n\n*Published*: {release_date}"

        cursor.execute(
            "INSERT OR REPLACE INTO docker_versions (repo, digest) VALUES (?, ?)",
            (app_name, digest_number),
        )
        conn.commit()

        message = {
            "blocks": [
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": f"{message}"
                    },
                    "accessory": {
                        "type": "button",
                        "text": {
                            "type": "plain_text",
                            "text": "🔗 Changelog url"
                        },
                        "url": f"{app_url}",
                        "action_id": "button-action"
                    }
                },
                {
                    "type": "divider"
                }
            ]
        }
        headers = {
            "Content-Type": "application/json"
        }
        response = requests.post(webhook_url, json=message, headers=headers)
        if 200 <= response.status_code < 300:
            logger.info(f"Message sent to Slack for {app_name}")
        else:
            logger.error(f"Failed to send message to Slack. Status code: {response.status_code}")
            logger.error(f"Response: {response.text}")
    conn.close()

