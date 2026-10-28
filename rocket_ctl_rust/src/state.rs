use pyo3::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::frame::TelemetryFrame;
use crate::constants::*;

#[pyclass(get_all, set_all)]
#[derive(Clone, Debug)]
pub struct RocketState {
    pub connected: bool,
    pub last_packet_time: f64,

    pub fuel_level: f64,
    pub oxidizer_level: f64,
    pub oxidizer_pressure: f64,
    pub altitude: f64,
    pub velocity: f64,
    pub angle: f64,
    pub max_altitude: f64,
    pub max_velocity: f64,

    pub servos: std::collections::HashMap<u8, i32>,
    pub relays: std::collections::HashMap<u8, bool>,

    pub apogee_detected: bool,
    pub parachute_deployed: bool,
    pub flight_stage: String,

    pub is_destroyed: bool,
    pub destruction_reason: String,

    pub step_ox_filled: bool,
    pub step_fuel_filled: bool,
    pub step_pressurized: bool,
    pub step_ignition: bool,
    pub step_vmax_reached: bool,
    pub step_hmax_reached: bool,
    pub step_parachute_deployed: bool,
    pub step_landed: bool,

    pub prev_altitude: f64,
    pub last_alt_time: f64,
}

#[pymethods]
impl RocketState {
    #[new]
    pub fn new() -> Self {
        let now = Instant::now().elapsed().as_secs_f64() + (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs_f64());
        let mut servos = HashMap::new();
        servos.insert(SERVO_FUEL_INTAKE, 100);
        servos.insert(SERVO_OXIDIZER_INTAKE, 100);
        servos.insert(SERVO_FUEL_MAIN, 100);
        servos.insert(SERVO_OXIDIZER_MAIN, 100);
        let mut relays = HashMap::new();
        relays.insert(RELAY_HEATER, false);
        relays.insert(RELAY_IGNITER, false);
        relays.insert(RELAY_PARACHUTE, false);
        Self {
            connected: false,
            last_packet_time: 0.0,
            fuel_level: 0.0,
            oxidizer_level: 0.0,
            oxidizer_pressure: 0.0,
            altitude: 0.0,
            velocity: 0.0,
            angle: 0.0,
            max_altitude: 0.0,
            max_velocity: 0.0,
            servos,
            relays,
            apogee_detected: false,
            parachute_deployed: false,
            flight_stage: "PRE-LAUNCH".to_string(),
            is_destroyed: false,
            destruction_reason: "".to_string(),
            step_ox_filled: false,
            step_fuel_filled: false,
            step_pressurized: false,
            step_ignition: false,
            step_vmax_reached: false,
            step_hmax_reached: false,
            step_parachute_deployed: false,
            step_landed: false,
            prev_altitude: 0.0,
            last_alt_time: now,
        }
    }

    fn reset(&mut self) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs_f64();
        self.fuel_level = 0.0;
        self.oxidizer_level = 0.0;
        self.oxidizer_pressure = 0.0;
        self.altitude = 0.0;
        self.velocity = 0.0;
        self.angle = 0.0;
        self.max_altitude = 0.0;
        self.max_velocity = 0.0;
        let mut servos = HashMap::new();
        servos.insert(SERVO_FUEL_INTAKE, 100);
        servos.insert(SERVO_OXIDIZER_INTAKE, 100);
        servos.insert(SERVO_FUEL_MAIN, 100);
        servos.insert(SERVO_OXIDIZER_MAIN, 100);
        self.servos = servos;
        let mut relays = HashMap::new();
        relays.insert(RELAY_HEATER, false);
        relays.insert(RELAY_IGNITER, false);
        relays.insert(RELAY_PARACHUTE, false);
        self.relays = relays;
        self.apogee_detected = false;
        self.parachute_deployed = false;
        self.flight_stage = "PRE-LAUNCH".to_string();
        self.is_destroyed = false;
        self.destruction_reason = "".to_string();
        self.step_ox_filled = false;
        self.step_fuel_filled = false;
        self.step_pressurized = false;
        self.step_ignition = false;
        self.step_vmax_reached = false;
        self.step_hmax_reached = false;
        self.step_parachute_deployed = false;
        self.step_landed = false;
        self.prev_altitude = 0.0;
        self.last_alt_time = now;
    }

    pub fn update_from_telemetry(&mut self, frame: &TelemetryFrame) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs_f64();
        self.last_packet_time = now;

        if frame.device_type == 2 {
            // sensor
            match frame.device_id {
                0 => {
                    if let Some(v) = frame.value_float {
                        let val = v as f64;
                        self.fuel_level = val;
                        if val >= 99.9 {
                            self.step_fuel_filled = true;
                        }
                        if self.flight_stage == "PRE-LAUNCH" && val < 99.0 && self.altitude > 10.0 {
                            self.flight_stage = "ASCENT".to_string();
                            self.step_ignition = true;
                        } else if self.flight_stage == "ASCENT" && val <= 0.1 && !self.apogee_detected {
                            self.flight_stage = "COASTING".to_string();
                            self.step_vmax_reached = true;
                        }
                    }
                }
                1 => {
                    if let Some(v) = frame.value_float {
                        let val = v as f64;
                        self.oxidizer_level = val;
                        if val >= 99.9 {
                            self.step_ox_filled = true;
                        }
                    }
                }
                2 => {
                    if let Some(v) = frame.value_float {
                        let val = v as f64;
                        self.altitude = val;

                        let dt = now - self.last_alt_time;
                        if dt >= 0.1 {
                            self.velocity = (val - self.prev_altitude) / dt;
                            self.prev_altitude = val;
                            self.last_alt_time = now;
                            if self.velocity > self.max_velocity {
                                self.max_velocity = self.velocity;
                            }
                        }

                        if val > self.max_altitude {
                            self.max_altitude = val;
                        }

                        if val > 2.0 || self.velocity > 1.0 {
                            self.step_ignition = true;
                        }

                        if (self.flight_stage == "PRE-LAUNCH" || self.flight_stage == "ASCENT" || self.flight_stage == "COASTING")
                            && self.max_altitude > 30.0
                            && (self.velocity <= 0.0 || val < self.max_altitude - 1.0)
                            && !self.apogee_detected
                        {
                            self.apogee_detected = true;
                            self.step_ignition = true;
                            self.step_vmax_reached = true;
                            self.step_hmax_reached = true;
                            self.flight_stage = "APOGEE REACHED".to_string();
                        }

                        if self.apogee_detected && (self.velocity < -0.5 || val < self.max_altitude - 2.0) {
                            self.step_ignition = true;
                            self.step_vmax_reached = true;
                            self.step_hmax_reached = true;
                            if self.parachute_deployed {
                                self.flight_stage = "DESCENT (PARACHUTE OPEN)".to_string();
                            } else {
                                self.flight_stage = "DESCENT (FREEFALL)".to_string();
                            }
                        }

                        if (self.apogee_detected || self.max_altitude > 30.0) && (val <= 1.5 || (self.parachute_deployed && val <= 10.0)) {
                            if self.parachute_deployed && !self.is_destroyed {
                                self.flight_stage = "LANDED".to_string();
                                self.step_landed = true;
                                self.altitude = 0.0;
                                self.velocity = 0.0;
                            }
                        }
                    }
                }
                3 => {
                    if let Some(v) = frame.value_float {
                        let val = v as f64;
                        self.oxidizer_pressure = val;
                        if val >= 55.0 {
                            self.step_pressurized = true;
                        }
                        if val >= 90.0 {
                            self.is_destroyed = true;
                            self.destruction_reason = "Tank Overpressure Explosion (>= 90 bar)".to_string();
                            self.flight_stage = "DESTROYED".to_string();
                        }
                    }
                }
                4 => {
                    if let Some(v) = frame.value_float {
                        self.angle = v as f64;
                    }
                }
                _ => {}
            }
        } else if frame.device_type == 0 {
            // servo
            if let Some(v) = frame.value_int {
                self.servos.insert(frame.device_id, v as i32);
            }
        } else if frame.device_type == 1 && frame.action == ACTION_ACK {
            // relay ack
            self.relays.insert(frame.device_id, frame.operation == 1);
            if frame.device_id == RELAY_PARACHUTE && frame.operation == 1 {
                self.parachute_deployed = true;
                self.step_parachute_deployed = true;
                self.flight_stage = "DESCENT (PARACHUTE OPEN)".to_string();
            }
        }
    }

    #[getter]
    fn can_refuel_fuel(&self) -> bool {
        self.oxidizer_level >= 99.9
    }

    #[getter]
    fn can_launch(&self) -> bool {
        let tanks_full = self.fuel_level >= 99.0 && self.oxidizer_level >= 99.0;
        let intakes_closed = *self.servos.get(&SERVO_FUEL_INTAKE).unwrap_or(&100) == 100
            && *self.servos.get(&SERVO_OXIDIZER_INTAKE).unwrap_or(&100) == 100;
        let pressure_safe = self.oxidizer_pressure >= 55.0 && self.oxidizer_pressure <= 65.0;
        tanks_full && intakes_closed && pressure_safe
    }

    #[getter]
    fn can_deploy_parachute(&self) -> bool {
        let engine_off = self.fuel_level <= 0.1;
        let safe_speed = self.velocity.abs() <= 28.0;
        let descending = self.apogee_detected || self.velocity < 0.0;
        engine_off && safe_speed && descending && !self.parachute_deployed
    }
}
