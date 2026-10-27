#!/bin/bash

# go to the project's root directory (where launch.sh is located)
cd "$(dirname -- "$(readlink -f "${BASH_SOURCE}")")"

# create a virtual environment if it doesn't exist
python3 -m venv venv/

# activate the virtual environment
. venv/bin/activate

echo "Installing Python dependencies..."
pip install --upgrade pip
pip install -r requirements.txt

maturin develop --release

echo
echo "Build completed successfully."
echo "Ready to start with ./launch.sh"
