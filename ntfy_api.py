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
            return jsonify({"error": f"The repo {repo} is already in the database."}), 409

        # Ajouter le dépôt à la base de données
        cursor.execute("INSERT INTO watched_repos (repo) VALUES (?)", (repo,))
        conn.commit()
        return jsonify({"message": f"The repo {repo} as been added to the watched repos."})
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


if __name__ == "__main__":
    app.run(debug=False)
