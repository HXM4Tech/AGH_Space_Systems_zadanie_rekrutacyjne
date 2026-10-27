import time
from typing import Dict

# Device IDs matching simulator_config.yaml
SERVO_FUEL_INTAKE = 0
SERVO_OXIDIZER_INTAKE = 1
SERVO_FUEL_MAIN = 2
SERVO_OXIDIZER_MAIN = 3

RELAY_HEATER = 0
RELAY_IGNITER = 1
RELAY_PARACHUTE = 2

SENSOR_MAP = {
    0: 'fuel_level',
    1: 'oxidizer_level',
    2: 'altitude',
    3: 'oxidizer_pressure',
    4: 'angle',
}

class RocketState:
    def __init__(self):
        self.connected: bool = False
        self.last_packet_time: float = 0.0

        # Telemetry
        self.fuel_level: float = 0.0
        self.oxidizer_level: float = 0.0
        self.oxidizer_pressure: float = 0.0
        self.altitude: float = 0.0
        self.velocity: float = 0.0
        self.angle: float = 0.0
        self.max_altitude: float = 0.0

        # Internal derivative calculation
        self._prev_altitude: float = 0.0
        self._last_alt_time: float = time.time()

        # Actuators
        self.servos: Dict[int, int] = {
            SERVO_FUEL_INTAKE: 100,
            SERVO_OXIDIZER_INTAKE: 100,
            SERVO_FUEL_MAIN: 100,
            SERVO_OXIDIZER_MAIN: 100,
        }
        self.relays: Dict[int, bool] = {
            RELAY_HEATER: False,
            RELAY_IGNITER: False,
            RELAY_PARACHUTE: False,
        }

        # Flight flags
        self.apogee_detected: bool = False
        self.parachute_deployed: bool = False
        self.flight_stage: str = "PRE-LAUNCH"

        # Destruction tracking
        self.is_destroyed: bool = False
        self.destruction_reason: str = ""

        # Maximum velocity tracked
        self.max_velocity: float = 0.0

        # Persistent checklist milestones
        self.step_ox_filled: bool = False
        self.step_fuel_filled: bool = False
        self.step_pressurized: bool = False
        self.step_ignition: bool = False
        self.step_vmax_reached: bool = False
        self.step_hmax_reached: bool = False
        self.step_parachute_deployed: bool = False
        self.step_landed: bool = False

    def reset(self):
        """Reset state and checklist milestones."""
        self.fuel_level = 0.0
        self.oxidizer_level = 0.0
        self.oxidizer_pressure = 0.0
        self.altitude = 0.0
        self.velocity = 0.0
        self.angle = 0.0
        self.max_altitude = 0.0
        self.max_velocity = 0.0
        self._prev_altitude = 0.0
        self._last_alt_time = time.time()
        self.servos = {
            SERVO_FUEL_INTAKE: 100,
            SERVO_OXIDIZER_INTAKE: 100,
            SERVO_FUEL_MAIN: 100,
            SERVO_OXIDIZER_MAIN: 100,
        }
        self.relays = {
            RELAY_HEATER: False,
            RELAY_IGNITER: False,
            RELAY_PARACHUTE: False,
        }
        self.apogee_detected = False
        self.parachute_deployed = False
        self.flight_stage = "PRE-LAUNCH"
        self.is_destroyed = False
        self.destruction_reason = ""
        self.step_ox_filled = False
        self.step_fuel_filled = False
        self.step_pressurized = False
        self.step_ignition = False
        self.step_vmax_reached = False
        self.step_hmax_reached = False
        self.step_parachute_deployed = False
        self.step_landed = False

    def update_from_telemetry(self, frame):
        """Update state from parsed Rust TelemetryFrame."""
        now = time.time()
        self.last_packet_time = now

        # Sensor FEED (device_type == 2)
        if frame.device_type == 2 and frame.device_id in SENSOR_MAP:
            name = SENSOR_MAP[frame.device_id]
            if frame.value_float is not None:
                val = float(frame.value_float)
                setattr(self, name, val)

                if name == 'altitude':
                    dt = now - self._last_alt_time
                    if dt >= 0.1:
                        self.velocity = (val - self._prev_altitude) / dt
                        self._prev_altitude = val
                        self._last_alt_time = now
                        if self.velocity > self.max_velocity:
                            self.max_velocity = self.velocity

                    if val > self.max_altitude:
                        self.max_altitude = val

                    # Monotonic Flight Stage State Machine:
                    # PRE-LAUNCH -> ASCENT -> COASTING -> APOGEE -> DESCENT -> LANDED
                    if val > 2.0 or self.velocity > 1.0:
                        self.step_ignition = True

                    if self.flight_stage in ("PRE-LAUNCH", "ASCENT", "COASTING"):
                        if self.max_altitude > 30.0 and (self.velocity <= 0.0 or val < self.max_altitude - 1.0) and not self.apogee_detected:
                            self.apogee_detected = True
                            self.step_ignition = True
                            self.step_vmax_reached = True
                            self.step_hmax_reached = True
                            self.flight_stage = "APOGEE REACHED"

                    if self.apogee_detected and (self.velocity < -0.5 or val < self.max_altitude - 2.0):
                        self.step_ignition = True
                        self.step_vmax_reached = True
                        self.step_hmax_reached = True
                        if self.parachute_deployed:
                            self.flight_stage = "DESCENT (PARACHUTE OPEN)"
                        else:
                            self.flight_stage = "DESCENT (FREEFALL)"

                    # Landing evaluation
                    if (self.apogee_detected or self.max_altitude > 30.0) and (val <= 1.5 or (self.parachute_deployed and val <= 10.0)):
                        if self.parachute_deployed and not self.is_destroyed:
                            self.flight_stage = "LANDED"
                            self.step_landed = True

                            self.altitude = 0.0
                            self.velocity = 0.0

                elif name == 'fuel_level':
                    if val >= 99.9:
                        self.step_fuel_filled = True
                    # Only transition to ASCENT from PRE-LAUNCH, never override DESCENT or LANDED!
                    if self.flight_stage == "PRE-LAUNCH" and val < 99.0 and self.altitude > 10.0:
                        self.flight_stage = "ASCENT"
                        self.step_ignition = True
                    elif self.flight_stage == "ASCENT" and val <= 0.1 and not self.apogee_detected:
                        self.flight_stage = "COASTING"
                        self.step_vmax_reached = True

                elif name == 'oxidizer_level':
                    if val >= 99.9:
                        self.step_ox_filled = True

                elif name == 'oxidizer_pressure':
                    if val >= 55.0:
                        self.step_pressurized = True
                    if val >= 90.0:
                        self.is_destroyed = True
                        self.destruction_reason = "Tank Overpressure Explosion (>= 90 bar)"
                        self.flight_stage = "DESTROYED"

        # Servo FEED / ACK (device_type == 0)
        elif frame.device_type == 0:
            if frame.value_int is not None:
                self.servos[frame.device_id] = frame.value_int

        # Relay ACK (device_type == 1)
        elif frame.device_type == 1 and frame.action == 2:
            self.relays[frame.device_id] = (frame.operation == 1)
            if frame.device_id == RELAY_PARACHUTE and frame.operation == 1:
                self.parachute_deployed = True
                self.step_parachute_deployed = True
                self.flight_stage = "DESCENT (PARACHUTE OPEN)"

    @property
    def can_refuel_fuel(self) -> bool:
        # NOTE: For a safe launch, the oxidizer tank should be filled first,
        # but I want to give the user the opportunity to experiment and
        # to fail the mission manually
        
        return True


    @property
    def can_launch(self) -> bool:
        # There must be at least some fuel and oxidizer to launch

        # NOTE: There are other requirements for a safe launch but
        # I want to give the user the opportunity to experiment and
        # to fail the mission manually
        
        return self.fuel_level > 0 and self.oxidizer_level > 0

    @property
    def can_deploy_parachute(self) -> bool:
        # Parachute must not have been deployed yet

        # NOTE: There are other requirements for a safe parachute deployment, such as
        # engine shutdown and low velocity, but I want to give the user the opportunity
        # to experiment and to fail the mission manually
        
        return not self.parachute_deployed
