import asyncio
import time
from nicegui import ui

from rocket_ctl_rust import RocketManager, Autopilot, constants

BASE_CARD_CLS = "bg-slate-900 border p-4 rounded-xl shadow transition-all "

CHECKLIST_ITEMS = [
    ("Refuel oxidizer tank", "step_ox_filled"),
    ("Refuel fuel tank", "step_fuel_filled"),
    ("Pressurize oxidizer", "step_pressurized"),
    ("Ignition", "step_ignition"),
    ("V-max / Engine shutdown", "step_vmax_reached"),
    ("H-max", "step_hmax_reached"),
    ("Deploy Parachute", "step_parachute_deployed"),
    ("Successful landing", "step_landed"),
]

class MissionDashboard:
    def __init__(self, manager: RocketManager, autopilot: Autopilot = None):
        self.manager = manager
        self.state = manager.state

        self.autopilot = autopilot
        
        self.auto_task: asyncio.Task = None
        self.auto_status_text: str = None

        self.chk_rows: list = []
        self.chk_icons: list = []
        self.chk_labels: list = []

        self._build_ui()

    def _build_ui(self):
        with ui.column().classes("w-full max-w-7xl mx-auto p-4 space-y-4"):
            # Header
            with ui.row().classes("w-full justify-between items-center bg-slate-900 border border-slate-800 rounded-xl p-4 shadow-lg"):
                with ui.row().classes("items-center space-x-3"):
                    ui.icon("rocket_launch", size="2.5rem").classes("text-blue-500")
                    with ui.column().classes("gap-0"):
                        ui.label("AGH SPACE SYSTEMS RECRUITMENT CHALLENGE").classes("text-xs font-semibold text-blue-400 tracking-widest")
                        ui.label("ROCKET MISSION CONTROL").classes("text-xl font-black text-white")

                with ui.row().classes("items-center space-x-3"):
                    ui.button("Reset State", icon="restart_alt", on_click=self.reset_mission).props("flat dense color=grey").classes("text-xs text-slate-400")
                    # Proxy connection badge
                    self.conn_badge = ui.label("CONNECTING TO PROXY...").classes(
                        "px-3 py-1.5 text-xs font-mono font-bold rounded-lg bg-rose-600 text-white shadow animate-pulse"
                    )
                    # Rocket communication / Mission stage label (independent of proxy connection)
                    self.stage_label = ui.label("No connection with rocket").classes(
                        "text-sm font-bold text-rose-500 font-mono"
                    )

            # Status Cards
            with ui.grid().classes("grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 w-full"):
                # Oxidizer Card
                with ui.card().classes(BASE_CARD_CLS + "border-slate-800"):
                    with ui.row().classes("w-full justify-between items-center"):
                        ui.label("OXIDIZER TANK").classes("text-xs font-bold text-slate-400")
                        ui.icon("water_drop", size="1.2rem").classes("text-cyan-400")
                    self.ox_val = ui.label("0.0%").classes("text-3xl font-black text-white font-mono my-2")
                    self.ox_bar = ui.linear_progress(value=0.0, show_value=False).classes("h-3 rounded-full bg-slate-800")
                    with ui.row().classes("text-xs text-slate-400 mt-2 justify-between w-full"):
                        ui.label("Intake Valve:")
                        self.ox_intake_label = ui.label("CLOSED").classes("font-mono font-bold text-white")

                # Fuel Card
                with ui.card().classes(BASE_CARD_CLS + "border-slate-800"):
                    with ui.row().classes("w-full justify-between items-center"):
                        ui.label("FUEL TANK").classes("text-xs font-bold text-slate-400")
                        ui.icon("local_gas_station", size="1.2rem").classes("text-blue-400")
                    self.fuel_val = ui.label("0.0%").classes("text-3xl font-black text-white font-mono my-2")
                    self.fuel_bar = ui.linear_progress(value=0.0, show_value=False).classes("h-3 rounded-full bg-slate-800")
                    with ui.row().classes("text-xs text-slate-400 mt-2 justify-between w-full"):
                        ui.label("Intake Valve:")
                        self.fuel_intake_label = ui.label("CLOSED").classes("font-mono font-bold text-white")

                # Pressure Card
                self.press_card = ui.card().classes(BASE_CARD_CLS + "border-slate-800")
                with self.press_card:
                    with ui.row().classes("w-full justify-between items-center"):
                        ui.label("OXIDIZER PRESSURE").classes("text-xs font-bold text-slate-400")
                        ui.icon("speed", size="1.2rem").classes("text-amber-400")
                    self.press_val = ui.label("0.0 bar").classes("text-3xl font-black text-white font-mono my-2")
                    self.press_desc = ui.label("Target: 55 – 65 bar").classes("text-xs text-slate-400")
                    with ui.row().classes("text-xs text-slate-400 mt-2 justify-between w-full"):
                        ui.label("Tank Heater:")
                        self.heater_label = ui.label("OFF").classes("font-mono font-bold text-white")

                # Flight Dynamics & Recovery Status
                with ui.card().classes(BASE_CARD_CLS + "border-slate-800"):
                    with ui.row().classes("w-full justify-between items-center"):
                        ui.label("FLIGHT TELEMETRY").classes("text-xs font-bold text-slate-400")
                        ui.icon("moving", size="1.2rem").classes("text-purple-400")
                    with ui.row().classes("items-baseline space-x-2 my-1"):
                        self.alt_val = ui.label("0.0 m").classes("text-3xl font-black text-emerald-400 font-mono")
                        self.vel_val = ui.label("0.0 m/s").classes("text-sm font-mono text-slate-300")
                    with ui.row().classes("text-xs text-slate-400 mt-1 justify-between w-full"):
                        self.max_alt_label = ui.label("Max: 0.0 m").classes("font-mono")
                        self.angle_label = ui.label("Angle: 0.0°").classes("font-mono text-white")
                    with ui.row().classes("text-xs mt-1 justify-between w-full items-center"):
                        ui.label("Parachute:").classes("text-slate-400")
                        self.parachute_telemetry_label = ui.label("STOWED").classes("font-mono font-bold text-white")

            # Checklist & Controls Section
            with ui.grid().classes("grid-cols-1 lg:grid-cols-2 gap-4 w-full items-start"):
                # Left Column: Mission Checklist
                with ui.card().classes("w-full bg-slate-900 border border-slate-800 rounded-xl p-5 shadow-lg space-y-4"):
                    # Switch & Real-time status info header
                    with ui.row().classes("w-full justify-between items-center pb-3 border-b border-slate-800"):
                        ui.label("FLIGHT CHECKLIST").classes("text-xs font-black text-slate-400 tracking-wider")

                        self.auto_info_badge = ui.label("STANDBY (MANUAL FLIGHT)").classes(
                            "text-xs font-mono font-bold px-3 py-1 rounded bg-slate-800 text-slate-400 border border-slate-700 transition-all"
                        )

                    # Checklist items
                    with ui.column().classes("w-full space-y-2"):
                        for idx, (title, _) in enumerate(CHECKLIST_ITEMS, start=1):
                            with ui.element("div").classes(
                                "w-full flex flex-row items-center gap-3 py-2.5 px-3 rounded-lg bg-slate-950/60 border border-slate-800 transition-all"
                            ) as row:
                                icon = ui.icon("check_box_outline_blank", size="1.25rem").classes("text-slate-600 shrink-0")
                                label = ui.label(f"{idx}. {title}").classes("text-sm text-slate-400 m-0 leading-none")
                                self.chk_rows.append(row)
                                self.chk_icons.append(icon)
                                self.chk_labels.append(label)

                # Right Column: Manual Flight Controls, Autopilot Toggle & Copyright notice
                with ui.column():
                    with ui.card().classes("w-full bg-slate-900 border border-slate-800 rounded-xl p-5 shadow-lg space-y-4"):
                        with ui.row().classes("w-full justify-between items-center pb-3 border-b border-slate-800"):
                            ui.label("MANUAL FLIGHT CONTROLS").classes("text-xs font-black text-slate-400 tracking-wider")
                            self.control_mode_badge = ui.label("MANUAL UNLOCKED").classes(
                                "text-xs font-mono px-2.5 py-1 rounded bg-emerald-950 text-emerald-400 border border-emerald-800 font-bold transition-all"
                            )

                        with ui.column().classes("w-full space-y-3"):
                            # Refueling Valves
                            with ui.grid().classes("grid-cols-2 sm:grid-cols-4 gap-2 w-full"):
                                self.btn_ox_refuel = ui.button(
                                    "Refuel Oxidizer",
                                    on_click=self.manager.refuel_oxidizer
                                ).classes("bg-cyan-700 hover:bg-cyan-600 text-white font-bold py-2 text-xs rounded-lg shadow")

                                self.btn_ox_close = ui.button(
                                    "Close Oxidizer",
                                    on_click=self.manager.close_oxidizer
                                ).classes("bg-slate-700 hover:bg-slate-600 text-white font-bold py-2 text-xs rounded-lg shadow")

                                self.btn_fuel_refuel = ui.button(
                                    "Refuel Fuel",
                                    on_click=self.manager.refuel_fuel
                                ).classes("bg-blue-700 hover:bg-blue-600 text-white font-bold py-2 text-xs rounded-lg shadow")

                                self.btn_fuel_close = ui.button(
                                    "Close Fuel",
                                    on_click=self.manager.close_fuel
                                ).classes("bg-slate-700 hover:bg-slate-600 text-white font-bold py-2 text-xs rounded-lg shadow")

                            # Tank Heating & Pressurization
                            with ui.grid().classes("grid-cols-2 gap-2 w-full"):
                                self.btn_heater_on = ui.button(
                                    "Heater ON",
                                    on_click=self.manager.activate_heater
                                ).classes("bg-amber-600 hover:bg-amber-500 text-white font-bold py-2 text-xs rounded-lg shadow")

                                self.btn_heater_off = ui.button(
                                    "Heater OFF",
                                    on_click=self.manager.deactivate_heater
                                ).classes("bg-slate-700 hover:bg-slate-600 text-white font-bold py-2 text-xs rounded-lg shadow")

                            # Main Ignition Sequence
                            with ui.row().classes("w-full"):
                                self.launch_btn = ui.button(
                                    "START (IGNITION SEQUENCE)",
                                    on_click=self.manager.ignition_sequence
                                ).classes("w-full bg-rose-600 hover:bg-rose-500 text-white font-bold py-2.5 text-xs rounded-lg shadow")

                            # Parachute Deployment
                            with ui.row().classes("w-full"):
                                self.parachute_btn = ui.button(
                                    "DEPLOY PARACHUTE",
                                    on_click=self.manager.deploy_parachute
                                ).classes("w-full bg-slate-700 text-slate-300 font-bold py-2.5 text-xs rounded-lg shadow")

                    with ui.card().classes("w-full bg-slate-900 border border-slate-800 rounded-xl p-5 shadow-lg space-y-4"):
                        ui.label("AUTOPILOT MODE").classes("text-xs font-black text-slate-400 tracking-wider")

                        with ui.row().classes("w-full items-center"):
                            self.auto_switch = ui.switch(value=False, on_change=self.on_auto_switch_change).props("color=cyan")
                            ui.label("Enable fully automated flight sequence").classes("text-slate-400")

                    with ui.card().classes("w-full bg-slate-900 border border-slate-800 rounded-xl p-5 shadow-lg items-center"):
                        ui.label("© 2026 Krzysztof Stefańczyk").classes("text-xs font-black text-slate-400 tracking-wide")
                        ui.link("GitHub Repository", "https://github.com/HXM4Tech/AGH_Space_Systems_zadanie_rekrutacyjne").classes("text-xs font-semibold text-blue-400 tracking-wide")

    def reset_mission(self):
        """Allows resetting state and checklist for repeated flight tests."""
        self.stop_autopilot()
        self.state.reset()

    def on_auto_switch_change(self, e):
        """Called when user toggles the autopilot switch."""
        if e.value:
            self.start_autopilot()
        else:
            self.stop_autopilot()

    def start_autopilot(self):
        """Starts autopilot flight execution."""
        if self.autopilot is None:
            return

        self.auto_task = self.autopilot.run_mission()

    def stop_autopilot(self):
        """Disengages autopilot, unblocking manual control."""
        if self.auto_task and not self.auto_task.done():
            try:
                self.auto_task.cancel()
            except Exception:
                pass

        if self.auto_switch.value:
            self.auto_switch.value = False

        self.auto_status_text = None

        res = self.autopilot.stop()
        asyncio.ensure_future(res)

    def update(self):
        """Called periodically by ui.timer to refresh dashboard state in real time."""
        now = time.time()

        # Touchdown evaluation fallback
        if (self.state.apogee_detected or self.state.max_altitude > 30.0) and not self.state.step_landed and not self.state.is_destroyed:
            if self.state.parachute_deployed and (self.state.altitude <= 1.5 or (self.state.altitude <= 25.0 and now - self.state.last_packet_time > 1.5)):
                self.state.flight_stage = "LANDED"
                self.state.step_landed = True

        # Connection status to TCP Proxy
        if self.state.connected:
            self.conn_badge.set_text("CONNECTED TO PROXY (127.0.0.1:3000)")
            self.conn_badge.classes(replace="px-3 py-1.5 text-xs font-mono font-bold rounded-lg bg-emerald-600 text-black shadow-md")
        else:
            self.conn_badge.set_text("CONNECTING TO PROXY... (127.0.0.1:3000)")
            self.conn_badge.classes(replace="px-3 py-1.5 text-xs font-mono font-bold rounded-lg bg-rose-600 text-white shadow-md animate-pulse")

        # Connection with rocket & mission stage
        has_rocket_comm = (self.state.last_packet_time > 0 and (now - self.state.last_packet_time <= 2.5))

        if has_rocket_comm:
            if self.state.step_landed or self.state.flight_stage == "LANDED":
                self.stage_label.set_text("MISSION STATUS: SUCCESSFUL")
                self.stage_label.classes(replace="text-sm font-bold text-emerald-400 font-mono")
            else:
                self.stage_label.set_text(f"MISSION STAGE: {self.state.flight_stage}")
                self.stage_label.classes(replace="text-sm font-bold text-amber-400 font-mono")
        else:
            # Telemetry from rocket is not arriving
            if self.state.step_landed or self.state.flight_stage == "LANDED":
                self.stage_label.set_text("MISSION STATUS: SUCCESSFUL")
                self.stage_label.classes(replace="text-sm font-bold text-emerald-400 font-mono")
            else:
                self.stage_label.set_text("No connection with rocket")
                self.stage_label.classes(replace="text-sm font-bold text-rose-500 font-mono")

        # Tanks indicators
        self.ox_val.set_text(f"{self.state.oxidizer_level:.1f}%")
        self.ox_bar.set_value(self.state.oxidizer_level / 100.0)

        self.ox_intake_label.set_text(
            "OPEN" if self.state.servos[constants.SERVO_OXIDIZER_INTAKE] == 0
            else "CLOSED"
        )

        self.fuel_val.set_text(f"{self.state.fuel_level:.1f}%")
        self.fuel_bar.set_value(self.state.fuel_level / 100.0)

        self.fuel_intake_label.set_text(
            "OPEN" if self.state.servos[constants.SERVO_FUEL_INTAKE] == 0
            else "CLOSED"
        )

        # Pressure indicator
        p = self.state.oxidizer_pressure
        self.press_val.set_text(f"{p:.1f} bar")

        self.heater_label.set_text(
            "ON" if self.state.relays[constants.RELAY_HEATER]
            else "OFF"
        )

        if 55.0 <= p <= 65.0:
            self.press_card.classes(replace=BASE_CARD_CLS + "bg-emerald-950/40 border-emerald-500")
            self.press_desc.set_text("OPTIMAL PRESSURE (100% THRUST)")
            self.press_desc.classes(replace="text-emerald-400 font-bold")
        elif p > 65.0:
            self.press_card.classes(replace=BASE_CARD_CLS + "bg-rose-950/40 border-rose-500")
            self.press_desc.set_text("DANGER! PRESSURE TOO HIGH (>65 bar)")
            self.press_desc.classes(replace="text-rose-400 font-bold")
        else:
            self.press_card.classes(replace=BASE_CARD_CLS + "border-slate-800")
            self.press_desc.set_text("Target: 55 – 65 bar")
            self.press_desc.classes(replace="text-slate-400")

        # Dynamics telemetry
        self.alt_val.set_text(f"{self.state.altitude:.1f} m")
        self.vel_val.set_text(f"{self.state.velocity:.1f} m/s")
        self.max_alt_label.set_text(f"Max: {self.state.max_altitude:.1f} m")
        self.angle_label.set_text(f"Angle: {self.state.angle:.1f}°")

        # Parachute status
        if self.state.parachute_deployed:
            self.parachute_telemetry_label.set_text("DEPLOYED")
            self.parachute_telemetry_label.classes(replace="font-mono font-bold text-emerald-400")
        else:
            self.parachute_telemetry_label.set_text("STOWED")
            self.parachute_telemetry_label.classes(replace="font-mono font-bold text-white")

        # Mission complete / control mode
        if self.state.step_landed:
            self.auto_info_badge.set_text("MISSION COMPLETE")
            self.auto_info_badge.classes(replace="text-xs font-mono font-bold px-3 py-1 rounded bg-emerald-600 text-white border border-emerald-500")
        elif self.autopilot.is_running():
            if self.auto_status_text is None:
                self.auto_info_badge.set_text("AUTOPILOT ACTIVE")
            else:
                self.auto_info_badge.set_text(f"AUTOPILOT ACTIVE: {self.auto_status_text}")
            
            self.auto_info_badge.classes(replace="text-xs font-mono font-bold px-3 py-1 rounded bg-blue-600 text-white border border-blue-500 animate-pulse")
        else:
            self.auto_info_badge.set_text("MANUAL FLIGHT")
            self.auto_info_badge.classes(replace="text-xs font-mono font-bold px-3 py-1 rounded bg-slate-800 text-slate-400 border border-slate-700")

        # Checklist items update
        checklist_status = [
            self.state.step_ox_filled,
            self.state.step_fuel_filled,
            self.state.step_pressurized,
            self.state.step_ignition,
            self.state.step_vmax_reached,
            self.state.step_hmax_reached,
            self.state.step_parachute_deployed,
            self.state.step_landed,
        ]

        for i, done in enumerate(checklist_status):
            row = self.chk_rows[i]
            icon = self.chk_icons[i]
            lbl = self.chk_labels[i]

            if done:
                row.classes(replace="w-full flex flex-row items-center gap-3 py-2.5 px-3 rounded-lg bg-emerald-950/20 border border-emerald-900/40 transition-all")
                icon._props["name"] = "check_box"
                icon.classes(replace="text-emerald-400 shrink-0 text-lg")
                icon.update()
                lbl.classes(replace="text-sm font-semibold text-emerald-100 m-0 leading-none")
            else:
                row.classes(replace="w-full flex flex-row items-center gap-3 py-2.5 px-3 rounded-lg bg-slate-950/60 border border-slate-800 transition-all")
                icon._props["name"] = "check_box_outline_blank"
                icon.classes(replace="text-slate-600 shrink-0 text-lg")
                icon.update()
                lbl.classes(replace="text-sm text-slate-400 font-normal m-0 leading-none")

        # Manual control buttons state (Disabled & grayed out if Autopilot is active)
        if self.autopilot.is_running():
            self.btn_ox_refuel.set_enabled(False)
            self.btn_ox_close.set_enabled(False)
            self.btn_fuel_refuel.set_enabled(False)
            self.btn_fuel_close.set_enabled(False)
            self.btn_heater_on.set_enabled(False)
            self.btn_heater_off.set_enabled(False)
            self.launch_btn.set_enabled(False)
            self.parachute_btn.set_enabled(False)
            self.parachute_btn.classes(replace="w-full bg-slate-800 text-slate-500 font-bold py-2.5 text-xs rounded-lg shadow")

            self.control_mode_badge.set_text("LOCKED (AUTOPILOT ACTIVE)")
            self.control_mode_badge.classes(replace="text-xs font-mono px-2.5 py-1 rounded bg-amber-950 text-amber-400 border border-amber-800 font-bold")

        else:
            self.btn_ox_refuel.set_enabled(
                self.state.servos[constants.SERVO_OXIDIZER_INTAKE] != 0
            )
            self.btn_ox_close.set_enabled(
                self.state.servos[constants.SERVO_OXIDIZER_INTAKE] == 0
            )
            self.btn_fuel_refuel.set_enabled(
                self.state.can_refuel_fuel and self.state.servos[constants.SERVO_FUEL_INTAKE] != 0
            )
            self.btn_fuel_close.set_enabled(
                self.state.servos[constants.SERVO_FUEL_INTAKE] == 0
            )
            self.btn_heater_on.set_enabled(
                not self.state.relays[constants.RELAY_HEATER]
            )
            self.btn_heater_off.set_enabled(
                self.state.relays[constants.RELAY_HEATER]
            )

            self.launch_btn.set_enabled(self.state.can_launch)
            self.parachute_btn.set_enabled(self.state.can_deploy_parachute)
            self.parachute_btn.classes(replace="w-full bg-emerald-600 hover:bg-emerald-500 text-white font-bold py-2.5 text-xs rounded-lg shadow-lg animate-pulse")

            self.control_mode_badge.set_text("MANUAL MODE")
            self.control_mode_badge.classes(replace="text-xs font-mono px-2.5 py-1 rounded bg-emerald-950 text-emerald-400 border border-emerald-800 font-bold")
