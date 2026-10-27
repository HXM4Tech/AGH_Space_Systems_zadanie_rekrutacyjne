mod calc_crc;
mod frame;

use frame::{
    build_command_frame, build_full_frame, parse_raw_frame, TelemetryFrame,
    ACTION_ACK, ACTION_FEED, ACTION_NACK, ACTION_SERVICE, BOARD_ROCKET, BOARD_SOFTWARE,
    DEVICE_RELAY, DEVICE_SENSOR, DEVICE_SERVO, PRIORITY_HIGH, PRIORITY_LOW,
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
fn rocket_comm_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_frame, m)?)?;
    m.add_function(wrap_pyfunction!(build_servo_command, m)?)?;
    m.add_function(wrap_pyfunction!(build_relay_command, m)?)?;
    m.add_function(wrap_pyfunction!(build_raw_frame, m)?)?;
    m.add_class::<TelemetryFrame>()?;

    // Export protocol constants
    m.add("BOARD_SOFTWARE", BOARD_SOFTWARE)?;
    m.add("BOARD_ROCKET", BOARD_ROCKET)?;
    m.add("ACTION_FEED", ACTION_FEED)?;
    m.add("ACTION_SERVICE", ACTION_SERVICE)?;
    m.add("ACTION_ACK", ACTION_ACK)?;
    m.add("ACTION_NACK", ACTION_NACK)?;
    m.add("PRIORITY_HIGH", PRIORITY_HIGH)?;
    m.add("PRIORITY_LOW", PRIORITY_LOW)?;
    m.add("DEVICE_SERVO", DEVICE_SERVO)?;
    m.add("DEVICE_RELAY", DEVICE_RELAY)?;
    m.add("DEVICE_SENSOR", DEVICE_SENSOR)?;

    Ok(())
}
