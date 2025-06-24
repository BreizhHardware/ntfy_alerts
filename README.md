<h1 align="center">Welcome to ntfy_alerts 👋</h1>
<p>
  <img alt="Version" src="https://img.shields.io/badge/version-2.1-blue.svg?cacheSeconds=2592000" />
  <a href="#" target="_blank">
    <img alt="License: GPL--3" src="https://img.shields.io/badge/License-GPL--3-yellow.svg" />
  </a>
  <a href="https://twitter.com/BreizhHardware" target="_blank">
    <img alt="Twitter: BreizhHardware" src="https://img.shields.io/twitter/follow/BreizhHardware.svg?style=social" />
  </a>
</p>

> This project allows you to receive notifications about new GitHub or Docker Hub releases on ntfy, gotify, Discord and Slack. Implemented in Rust for better performance.

## Installation

### Docker (recommended)

Use our Docker image, which automatically supports amd64, arm64 and armv7:

```yaml
services:
  github-ntfy:
    image: breizhhardware/github-ntfy:latest
    container_name: github-ntfy
    volumes:
      - /path/to/data:/data
    ports:
      - 80:80
    restart: unless-stopped
```

### Manual Installation
Install Rust if needed
```BASH
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone the repository
```BASH
git clone https://github.com/BreizhHardware/ntfy_alerts.git
cd ntfy_alerts
```

Compile
```BASH
cargo build --release
```

Run
```BASH
./target/release/github-ntfy
```

## Version Notes
- v2.0: Complete rewrite in Rust for better performance and reduced resource consumption
- [v1.7.1](https://github.com/BreizhHardware/ntfy_alerts/tree/v1.7.2): Stable Python version

## Configuration
The GitHub token (GHNTFY_TOKEN) needs to have the following permissions: repo, read:org and read:user.

## TODO
- [ ] Add support for multi achitecture Docker images
- [x] Rework web interface
- [ ] Add support for more notification services (Telegram, Matrix, etc.)
- [x] Add web oneboarding instead of using environment variables

## Author
👤 BreizhHardware


- Website: [https://mrqt.fr](https://mrqt.fr?ref=github)
- Twitter: [@BreizhHardware](https://twitter.com/BreizhHardware)
- Github: [@BreizhHardware](https://github.com/BreizhHardware)
- LinkedIn: [@félix-marquet-5071bb167](https://linkedin.com/in/félix-marquet-5071bb167)

## Contributing
Contributions are what make the open-source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**. But first, please read the [CONTRIBUTION.md](CONTRIBUTION.md) file.

## Show your support
Give a ⭐️ if this project helped you!