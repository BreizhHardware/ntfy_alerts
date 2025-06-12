FROM 1.87.0-alpine3.22 as builder

LABEL maintainer="BreizhHardware"
LABEL version_number="1.4"

WORKDIR /app

# Installation of dependencies
RUN apk add --no-cache sqlite-dev musl-dev openssl-dev pkgconfig

# Copy of the source files
COPY Cargo.toml Cargo.lock ./

# Create a temp source file to pre download dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy real file
COPY src/ ./src/

# Build the application
RUN cargo build --release

# Final image
FROM alpine:3.22

# Install of runtime dependencies
RUN apk add --no-cache sqlite-libs openssl nginx

# Copy the static files
COPY index.html /var/www/html/index.html
COPY script.js /var/www/html/script.js

# Copy the built application from the builder stage
COPY --from=builder /app/target/release/github-ntfy /usr/local/bin/github-ntfy

# Configure Nginx
COPY nginx.conf /etc/nginx/nginx.conf

# Copy the entrypoint script
COPY entrypoint.sh /
RUN chmod 700 /entrypoint.sh

# Define the working directory
ENV USERNAME="" \
    PASSWORD="" \
    NTFY_URL="" \
    GHNTFY_TIMEOUT="3600" \
    GHNTFY_TOKEN="" \
    DOCKER_USERNAME="" \
    DOCKER_PASSWORD="" \
    GOTIFY_URL="" \
    GOTIFY_TOKEN="" \
    DISCORD_WEBHOOK_URL="" \
    SLACK_WEBHOOK_URL=""

RUN mkdir -p /github-ntfy && chmod 755 /github-ntfy

EXPOSE 5000 80

ENTRYPOINT ["/entrypoint.sh"]