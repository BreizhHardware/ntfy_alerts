from flask import Flask, request, jsonify
from flask_cors import CORS
import sqlite3

app = Flask(__name__)
CORS(app)
app.logger.setLevel("WARNING")

def get_db_connection():
    conn = sqlite3.connect('/github-ntfy/watched_repos.db')
    conn.row_factory = sqlite3.Row
    return conn


def close_db_connection(conn):
    conn.close()


@app.route('/app_repo', methods=['POST'])
def app_repo():
    data = request.json
    repo = data.get('repo')

    # Vérifier si le champ 'repo' est présent dans les données JSON
    if not repo:
        return jsonify({"error": "The repo field is required."}), 400

    # Établir une connexion à la base de données
    conn = get_db_connection()
    cursor = conn.cursor()

    try:
        # Vérifier si le dépôt existe déjà dans la base de données
        cursor.execute("SELECT * FROM watched_repos WHERE repo=?", (repo,))
        existing_repo = cursor.fetchone()
        if existing_repo:
            return jsonify({"error": f"The GitHub repo {repo} is already in the database."}), 409

        # Ajouter le dépôt à la base de données
        cursor.execute("INSERT INTO watched_repos (repo) VALUES (?)", (repo,))
        conn.commit()
        return jsonify({"message": f"The GitHub repo {repo} as been added to the watched repos."})
    finally:
        # Fermer la connexion à la base de données
        close_db_connection(conn)


@app.route('/app_docker_repo', methods=['POST'])
def app_docker_repo():
    data = request.json
    repo = data.get('repo')

    # Vérifier si le champ 'repo' est présent dans les données JSON
    if not repo:
        return jsonify({"error": "The repo field is required."}), 400

    # Établir une connexion à la base de données
    conn = get_db_connection()
    cursor = conn.cursor()

    try:
        # Vérifier si le dépôt existe déjà dans la base de données
        cursor.execute("SELECT * FROM docker_watched_repos WHERE repo=?", (repo,))
        existing_repo = cursor.fetchone()
        if existing_repo:
            return jsonify({"error": f"The Docker repo {repo} is already in the database."}), 409

        # Ajouter le dépôt à la base de données
        cursor.execute("INSERT INTO docker_watched_repos (repo) VALUES (?)", (repo,))
        conn.commit()
        return jsonify({"message": f"The Docker repo {repo} as been added to the watched repos."})
    finally:
        # Fermer la connexion à la base de données
        close_db_connection(conn)


@app.route('/watched_repos', methods=['GET'])
def get_watched_repos():
    db = get_db_connection()
    cursor = db.cursor()
    cursor.execute("SELECT repo FROM watched_repos")
    watched_repos = [repo[0] for repo in cursor.fetchall()]
    cursor.close()
    db.close()
    return jsonify(watched_repos)


@app.route('/watched_docker_repos', methods=['GET'])
def get_watched_docker_repos():
    db = get_db_connection()
    cursor = db.cursor()
    cursor.execute("SELECT repo FROM docker_watched_repos")
    watched_repos = [repo[0] for repo in cursor.fetchall()]
    cursor.close()
    db.close()
    return jsonify(watched_repos)


@app.route('/delete_repo', methods=['POST'])
def delete_repo():
    data = request.json
    repo = data.get('repo')

    # Vérifier si le champ 'repo' est présent dans les données JSON
    if not repo:
        return jsonify({"error": "The repo field is required."}), 400

    # Établir une connexion à la base de données
    conn = get_db_connection()
    cursor = conn.cursor()

    try:
        # Vérifier si le dépôt existe dans la base de données
        cursor.execute("SELECT * FROM watched_repos WHERE repo=?", (repo,))
        existing_repo = cursor.fetchone()
        if not existing_repo:
            return jsonify({"error": f"The GitHub repo {repo} is not in the database."}), 404

        # Supprimer le dépôt de la base de données
        cursor.execute("DELETE FROM watched_repos WHERE repo=?", (repo,))
        conn.commit()
        return jsonify({"message": f"The GitHub repo {repo} as been deleted from the watched repos."})
    finally:
        # Fermer la connexion à la base de données
        close_db_connection(conn)


@app.route('/delete_docker_repo', methods=['POST'])
def delete_docker_repo():
    data = request.json
    repo = data.get('repo')

    # Vérifier si le champ 'repo' est présent dans les données JSON
    if not repo:
        return jsonify({"error": "The repo field is required."}), 400

    # Établir une connexion à la base de données
    conn = get_db_connection()
    cursor = conn.cursor()

    try:
        # Vérifier si le dépôt existe dans la base de données
        cursor.execute("SELECT * FROM docker_watched_repos WHERE repo=?", (repo,))
        existing_repo = cursor.fetchone()
        if not existing_repo:
            return jsonify({"error": f"The Docker repo {repo} is not in the database."}), 404

        # Supprimer le dépôt de la base de données
        cursor.execute("DELETE FROM docker_watched_repos WHERE repo=?", (repo,))
        conn.commit()
        return jsonify({"message": f"The Docker repo {repo} as been deleted from the watched repos."})
    finally:
        # Fermer la connexion à la base de données
        close_db_connection(conn)


if __name__ == "__main__":
    app.run(debug=False)
