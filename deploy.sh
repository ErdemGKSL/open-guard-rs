#!/bin/bash

# Exit on error
set -e

echo "Building project in release mode..."
cargo build -r

# Load environment variables from .env
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
else
    echo ".env file not found!"
    exit 1
fi

if [ -z "$DEPLOY_IP" ]; then
    echo "DEPLOY_IP not set in .env"
    exit 1
fi

# Configuration
PEM_FILE="$HOME/Documents/aws.pem"
REMOTE_TARGET="ec2-user@$DEPLOY_IP:~/openguard/"
BINARY_PATH="./target/release/open-guard-rs"

echo "Deploying binary to remote server..."
scp -i "$PEM_FILE" "$BINARY_PATH" "$REMOTE_TARGET"

echo "Deployment finished successfully!"
