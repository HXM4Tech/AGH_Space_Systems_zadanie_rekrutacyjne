#!/bin/bash

# go to the project's root directory (where launch.sh is located)
cd "$(dirname -- "$(readlink -f "${BASH_SOURCE}")")"

# activate the virtual environment
. venv/bin/activate

# set up trap to terminate tcp_proxy.py and tcp_simulator.py when the script exits
bgpids=()
killbg() {
    echo
    echo "Terminating background processes..."

    for p in "${bgpids[@]}" ; do
        kill "$p" 2>/dev/null;
    done
}

trap killbg EXIT

# create logs directory if it doesn't exist
mkdir -p logs

# start tcp_proxy.py and tcp_simulator.py in the background, redirecting their output to log files

python3 tcp_proxy.py > logs/tcp_proxy.log 2>&1 < /dev/null &
bgpids+=($!)

sleep 0.5

python3 tcp_simulator.py > logs/tcp_simulator.log 2>&1 < /dev/null &
bgpids+=($!)

echo "Started tcp_proxy.py and tcp_simulator.py in the background."
echo "Logs are being written to logs/tcp_proxy.log and logs/tcp_simulator.log."
echo
echo "Starting web_ui/main.py..."
echo

python3 web_ui/main.py
