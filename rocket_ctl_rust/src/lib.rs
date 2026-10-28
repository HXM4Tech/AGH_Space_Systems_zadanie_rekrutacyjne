mod autopilot;
mod checksum;
mod client;
mod constants;
mod frame;
mod manager;
mod state;

use crate::autopilot::Autopilot;
use crate::manager::RocketManager;
use crate::state::RocketState;
use crate::constants::*;

use pyo3::prelude::*;

#[pymodule]
fn rocket_ctl_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RocketState>()?;
    m.add_class::<RocketManager>()?;
    m.add_class::<Autopilot>()?;

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
