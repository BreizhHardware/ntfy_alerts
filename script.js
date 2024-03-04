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
                listItem.textContent = repo;
                watchedReposList.appendChild(listItem);
            });
        })
        .catch(error => {
            console.error('Error:', error);
        });
}

// Appeler la fonction pour charger les dépôts surveillés au chargement de la page
refreshWatchedRepos();