#!/usr/bin/env node
// Prepares the docker-build/ staging directory for the CI-style build
// (mirrors what .github/workflows/create_dev.yml does in the docker-build-push job)
import { cpSync, mkdirSync, rmSync } from 'fs'
import { join } from 'path'

const root = new URL('..', import.meta.url).pathname.replace(/^\/([A-Z]:)/, '$1')
const staging = join(root, 'docker-build')

rmSync(staging, { recursive: true, force: true })
mkdirSync(join(staging, 'web-output', 'public'), { recursive: true })

cpSync(
  join(root, 'target', 'x86_64-unknown-linux-musl', 'release', 'github-ntfy'),
  join(staging, 'github-ntfy')
)
cpSync(
  join(root, 'web', '.output', 'public'),
  join(staging, 'web-output', 'public'),
  { recursive: true }
)
for (const file of ['nginx.conf', 'entrypoint.sh', 'Dockerfile']) {
  cpSync(join(root, file), join(staging, file))
}

console.log('Docker build context prepared at docker-build/')
