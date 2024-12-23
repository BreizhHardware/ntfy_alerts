<h1 align="center">Welcome to ntfy_alerts 👋</h1>
<p>
  <img alt="Version" src="https://img.shields.io/badge/version-1.5-blue.svg?cacheSeconds=2592000" />
  <a href="#" target="_blank">
    <img alt="License: GPL--3" src="https://img.shields.io/badge/License-GPL--3-yellow.svg" />
  </a>
  <a href="https://twitter.com/BreizhHardware" target="_blank">
    <img alt="Twitter: BreizhHardware" src="https://img.shields.io/twitter/follow/BreizhHardware.svg?style=social" />
  </a>
</p>

> This project allows you to receive notifications about new GitHub or Docker Hub releases on ntfy, gotify, and Discord.

## Installation

To install the dependencies, run:
```sh
pip install -r requirements.txt
```

## Usage

If you want to use the Docker image, you can use the following docker-compose file for x86_64:
````yaml
services:
  github-ntfy:
    image: breizhhardware/github-ntfy:latest
    container_name: github-ntfy
    environment:
      - USERNAME=username # Required
      - PASSWORD=password # Required
      - NTFY_URL=ntfy_url # Required if ntfy is used
      - GHNTFY_TIMEOUT=timeout # Default is 3600 (1 hour)
      - GHNTFY_TOKEN= # Default is empty (Github token)
      - DOCKER_USERNAME= # Default is empty (Docker Hub username)
      - DOCKER_PASSWORD= # Default is empty (Docker Hub password)
      - GOTIFY_URL=gotify_url # Required if gotify is used
      - GOTIFY_TOKEN= # Required if gotify is used
      - DISCORD_WEBHOOK_URL= # Required if discord is used
    volumes:
      - /path/to/github-ntfy:/github-ntfy/
    ports:
      - 80:80
    restart: unless-stopped
````
For arm64 this docker compose file is ok:
````yaml
services:
  github-ntfy:
    image: breizhhardware/github-ntfy:arm64
    container_name: github-ntfy
    environment:
      - USERNAME=username # Required
      - PASSWORD=password # Required
      - NTFY_URL=ntfy_url # Required if ntfy is used
      - GHNTFY_TIMEOUT=timeout # Default is 3600 (1 hour)
      - GHNTFY_TOKEN= # Default is empty (Github token)
      - DOCKER_USERNAME= # Default is empty (Docker Hub username)
      - DOCKER_PASSWORD= # Default is empty (Docker Hub password)
      - GOTIFY_URL=gotify_url # Required if gotify is used
      - GOTIFY_TOKEN= # Required if gotify is used
      - DISCORD_WEBHOOK_URL= # Required if discord is used
    volumes:
      - /path/to/github-ntfy:/github-ntfy/
    ports:
      - 80:80
    restart: unless-stopped
````
For armV7 this docker compose is ok:
````yaml
services:
  github-ntfy:
    image: breizhhardware/github-ntfy:armv7
    container_name: github-ntfy
    environment:
      - USERNAME=username # Required
      - PASSWORD=password # Required
      - NTFY_URL=ntfy_url # Required if ntfy is used
      - GHNTFY_TIMEOUT=timeout # Default is 3600 (1 hour)
      - GHNTFY_TOKEN= # Default is empty (Github token)
      - DOCKER_USERNAME= # Default is empty (Docker Hub username)
      - DOCKER_PASSWORD= # Default is empty (Docker Hub password)
      - GOTIFY_URL=gotify_url # Required if gotify is used
      - GOTIFY_TOKEN= # Required if gotify is used
      - DISCORD_WEBHOOK_URL= # Required if discord is used
    volumes:
      - /path/to/github-ntfy:/github-ntfy/
    ports:
      - 80:80
    restart: unless-stopped
````
GHNTFY_TOKEN is a github token, it need to have repo, read:org and read:user

## Author

👤 **BreizhHardware**

* Website: https://mrqt.fr?ref=github
* Twitter: [@BreizhHardware](https://twitter.com/BreizhHardware)
* Github: [@BreizhHardware](https://github.com/BreizhHardware)
* LinkedIn: [@félix-marquet-5071bb167](https://linkedin.com/in/félix-marquet-5071bb167)

## Contribution

If you want to contribut, feel free to open a pull request, but first read the [contribution guide](CONTRIBUTION.md)!

## TODO:
- [x] Dockerize the ntfy.py
- [x] Add the watched repos list as a parameter
- [x] Add the application version as a database
- [x] Add the watched repos list as a web interface
- [x] Add Docker Hub compatibility
- [ ] Rework of the web interface
- [x] Compatibility with Gotify
- [x] Compatibility with Discord Webhook
- [x] Compatibility and distribution for arm64 and armv7

## Show your support

Give a ⭐️ if this project helped you!