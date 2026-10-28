mod calc_crc;
mod client;
mod manager;
mod frame;
mod state;
mod constants;

use manager::RocketManager;
use state::RocketState;

use constants::*;

use frame::{
    build_command_frame, build_full_frame, parse_raw_frame, TelemetryFrame
};

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyfunction]
fn parse_frame(data: &[u8]) -> PyResult<TelemetryFrame> {
    parse_raw_frame(data).map_err(|e| PyValueError::new_err(e))
}

#[pyfunction]
fn build_servo_command(device_id: u8, position: i16) -> PyResult<Vec<u8>> {
    let mut payload = [0u8; 4];
    payload[..2].copy_from_slice(&position.to_le_bytes());
    Ok(build_command_frame(BOARD_ROCKET, DEVICE_SERVO, device_id, 5, 5, payload))
}

#[pyfunction]
fn build_relay_command(device_id: u8, open: bool) -> PyResult<Vec<u8>> {
    let op = if open { 1 } else { 2 };
    Ok(build_command_frame(BOARD_ROCKET, DEVICE_RELAY, device_id, op, 0, [0u8; 4]))
}

#[pyfunction]
#[pyo3(signature = (destination, priority, action, source, device_type, device_id, data_type, operation, payload=None))]
fn build_raw_frame(
    destination: u8,
    priority: u8,
    action: u8,
    source: u8,
    device_type: u8,
    device_id: u8,
    data_type: u8,
    operation: u8,
    payload: Option<[u8; 4]>,
) -> PyResult<Vec<u8>> {
    let p = payload.unwrap_or([0u8; 4]);
    Ok(build_full_frame(
        destination,
        priority,
        action,
        source,
        device_type,
        device_id,
        data_type,
        operation,
        p,
    ))
}

#[pymodule]
fn rocket_ctl_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_frame, m)?)?;
    m.add_function(wrap_pyfunction!(build_servo_command, m)?)?;
    m.add_function(wrap_pyfunction!(build_relay_command, m)?)?;
    m.add_function(wrap_pyfunction!(build_raw_frame, m)?)?;
    m.add_class::<TelemetryFrame>()?;
    m.add_class::<RocketState>()?;
    m.add_class::<RocketManager>()?;

    let c = pyo3::types::PyModule::new(
        m.py(),
        "constants"
    )?;
    
    // Export protocol constants
    c.add("BOARD_SOFTWARE", BOARD_SOFTWARE)?;
    c.add("BOARD_ROCKET", BOARD_ROCKET)?;
    c.add("ACTION_FEED", ACTION_FEED)?;
    c.add("ACTION_SERVICE", ACTION_SERVICE)?;
    c.add("ACTION_ACK", ACTION_ACK)?;
    c.add("ACTION_NACK", ACTION_NACK)?;
    c.add("PRIORITY_HIGH", PRIORITY_HIGH)?;
    c.add("PRIORITY_LOW", PRIORITY_LOW)?;
    c.add("DEVICE_SERVO", DEVICE_SERVO)?;
    c.add("DEVICE_RELAY", DEVICE_RELAY)?;
    c.add("DEVICE_SENSOR", DEVICE_SENSOR)?;

    // Mission control constants
    c.add("SERVO_FUEL_INTAKE", SERVO_FUEL_INTAKE)?;
    c.add("SERVO_OXIDIZER_INTAKE", SERVO_OXIDIZER_INTAKE)?;
    c.add("SERVO_FUEL_MAIN", SERVO_FUEL_MAIN)?;
    c.add("SERVO_OXIDIZER_MAIN", SERVO_OXIDIZER_MAIN)?;
    c.add("RELAY_HEATER", RELAY_HEATER)?;
    c.add("RELAY_IGNITER", RELAY_IGNITER)?;
    c.add("RELAY_PARACHUTE", RELAY_PARACHUTE)?;

    m.add_submodule(&c)?;

    Ok(())
}
