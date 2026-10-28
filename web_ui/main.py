#!/usr/bin/env python3

import os
import sys
from nicegui import app, ui

# Ensure web_ui directory is in sys.path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import rocket_ctl_rust
from components import MissionDashboard

manager = rocket_ctl_rust.RocketManager(host="127.0.0.1", port=3000)
autopilot = rocket_ctl_rust.Autopilot(manager)

@ui.page('/')
def index():
    """Renders the dashboard for each connected browser session."""
    dashboard = MissionDashboard(manager, autopilot)
    ui.timer(0.1, dashboard.update)


app.on_startup(manager.connect)

if __name__ in {"__main__", "__mp_main__"}:
    try:
        ui.run(
            port=8080,
            title="AGH Space Systems Recruitment Challenge",
            dark=True,
            reload=False,
            show=False
        )
    except KeyboardInterrupt:
        print()
        print("Shutting down...")
        sys.exit(0)
