#!/bin/bash

# Install the latest version of Rustup.
echo "Installing Rustup..."

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Install the Diesel CLI.
echo "Make sure the PostgreSQL CLI is installed!"
echo "Installing Diesel CLI..."

cargo install diesel_cli --no-default-features --features postgres

# Start the database service.
echo "Starting the database service temporarily..."

docker compose up database -d
cd common || exit

# Create the database.
diesel migration run
cd .. || exit

# Stop the database service.
docker compose down

# Inform the user that the setup is complete.
echo "Setup complete, you can now run RSE!"
