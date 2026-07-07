#!/bin/sh

# Check if USERNAME and PASSWORD environment variables are defined
if [ -n "$USERNAME" ] && [ -n "$PASSWORD" ]; then
  # Generate auth.txt file content from environment variables
  echo -n "$USERNAME:$PASSWORD" > /auth.txt
  echo "Authentication file generated from environment variables"
else
  echo "USERNAME and/or PASSWORD variables not defined"
  echo "Authentication will be managed by the onboarding system via the web interface"
fi

# Set database directory permissions
if [ -d "/github-ntfy" ]; then
  chmod -R 755 /github-ntfy
  echo "Permissions applied to data directory"
fi

# Start nginx in the background
echo "Starting Nginx..."
nginx -g 'daemon off;' &

# Start the main application
echo "Starting application..."
exec /usr/local/bin/github-ntfy