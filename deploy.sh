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
REMOTE_DIR="/home/ec2-user/openguard"
REMOTE_USER="ec2-user"
BINARY_PATH="./target/release/open-guard-rs"

echo "Stopping open-guard service..."
ssh -i "$PEM_FILE" "$REMOTE_USER@$DEPLOY_IP" "sudo systemctl stop open-guard || true"

echo "Ensuring remote directory exists..."
ssh -i "$PEM_FILE" "$REMOTE_USER@$DEPLOY_IP" "mkdir -p $REMOTE_DIR"

echo "Deploying binary to remote server..."
scp -i "$PEM_FILE" "$BINARY_PATH" "$REMOTE_USER@$DEPLOY_IP:$REMOTE_DIR/open-guard-rs"

echo "Registering commands (publish)..."
ssh -i "$PEM_FILE" "$REMOTE_USER@$DEPLOY_IP" "cd $REMOTE_DIR && ./open-guard-rs --publish"

echo "Starting open-guard service..."
ssh -i "$PEM_FILE" "$REMOTE_USER@$DEPLOY_IP" "sudo systemctl start open-guard"

echo "Deployment finished successfully!"
