document.getElementById('addRepoForm').addEventListener('submit', function(event) {
    event.preventDefault();
    let repoName = document.getElementById('repo').value;
    fetch('/app_repo', {
        method: 'POST',
        headers: {
            'Access-Control-Allow-Origin': '*',
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({repo: repoName})
    })
        .then(response => {
            if (response.ok) {
                // Si la requête s'est bien déroulée, actualiser la liste des dépôts surveillés
                refreshWatchedRepos();
            } else {
                throw new Error('Erreur lors de l\'ajout du dépôt');
            }
        })
        .catch(error => {
            console.error('Error:', error);
        });
});

document.getElementById('addDockerRepoForm').addEventListener('submit', function(event) {
    event.preventDefault();
    let repoName = document.getElementById('dockerRepo').value;
    fetch('/app_docker_repo', {
        method: 'POST',
        headers: {
            'Access-Control-Allow-Origin': '*',
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({repo: repoName})
    })
        .then(response => {
            if (response.ok) {
                // Si la requête s'est bien déroulée, actualiser la liste des dépôts surveillés
                refreshWatchedRepos();
            } else {
                throw new Error('Erreur lors de l\'ajout du dépôt');
            }
        })
        .catch(error => {
            console.error('Error:', error);
        });
});

function refreshWatchedRepos() {
    fetch('/watched_repos')
        .then(response => response.json())
        .then(data => {
            const watchedReposList = document.getElementById('watchedReposList');
            // Vider la liste actuelle
            watchedReposList.innerHTML = '';
            // Ajouter chaque dépôt surveillé à la liste
            data.forEach(repo => {
                const listItem = document.createElement('li');
                const repoName = document.createElement('span');
                repoName.textContent = repo;
                repoName.className = 'repo-name';
                listItem.appendChild(repoName);

                const deleteButton = document.createElement('button');
                deleteButton.textContent = ' X';
                deleteButton.className = 'delete-btn text-red-500 ml-2';
                deleteButton.addEventListener('click', () => {
                    // Remove the repo from the watched repos
                    // This is a placeholder. Replace it with your actual code to remove the repo from the watched repos.
                    removeRepoFromWatchedRepos(repo);

                    // Remove the repo from the DOM
                    listItem.remove();
                });
                listItem.appendChild(deleteButton);

                watchedReposList.appendChild(listItem);
            });
        })
        .catch(error => {
            console.error('Error:', error);
        });

    fetch('/watched_docker_repos')
        .then(response => response.json())
        .then(data => {
            const watchedDockerReposList = document.getElementById('watchedDockerReposList');
            // Vider la liste actuelle
            watchedDockerReposList.innerHTML = '';
            // Ajouter chaque dépôt surveillé à la liste
            data.forEach(repo => {
                const listItem = document.createElement('li');
                const repoName = document.createElement('span');
                repoName.textContent = repo;
                repoName.className = 'repo-name';
                listItem.appendChild(repoName);

                const deleteButton = document.createElement('button');
                deleteButton.textContent = ' X';
                deleteButton.className = 'delete-btn text-red-500 ml-2';
                deleteButton.addEventListener('click', () => {
                    // Remove the repo from the watched repos
                    // This is a placeholder. Replace it with your actual code to remove the repo from the watched repos.
                    removeDockerRepoFromWatchedRepos(repo);

                    // Remove the repo from the DOM
                    listItem.remove();
                });
                listItem.appendChild(deleteButton);

                watchedDockerReposList.appendChild(listItem);
            });
        })
        .catch(error => {
            console.error('Error:', error);
        });
}

function removeRepoFromWatchedRepos(repo) {
    fetch('/delete_repo', {
        method: 'POST',
        headers: {
            'Access-Control-Allow-Origin': '*',
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({repo: repo})
    })
        .then(response => {
            if (!response.ok) {
                throw new Error('Erreur lors de la suppression du dépôt');
            }
        })
        .catch(error => {
            console.error('Error:', error);
        });
}

function removeDockerRepoFromWatchedRepos(repo) {
    fetch('/delete_docker_repo', {
        method: 'POST',
        headers: {
            'Access-Control-Allow-Origin': '*',
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({repo: repo})
    })
        .then(response => {
            if (!response.ok) {
                throw new Error('Erreur lors de la suppression du dépôt');
            }
        })
        .catch(error => {
            console.error('Error:', error);
        });
}

// Appeler la fonction pour charger les dépôts surveillés au chargement de la page
refreshWatchedRepos();