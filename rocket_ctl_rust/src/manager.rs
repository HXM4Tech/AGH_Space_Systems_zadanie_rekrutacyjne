use crate::client::RocketClient;
use crate::constants::*;

use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use core::time;
use std::sync::Arc;
use parking_lot::Mutex;

#[pyclass]
pub struct RocketManager {
    client: Arc<RocketClient>,
    state: Arc<Mutex<Py<PyAny>>>,
}

#[pymethods]
impl RocketManager {
    #[new]
    #[pyo3(signature = (state, host="127.0.0.1", port=3000))]
    fn new(state: Py<PyAny>, host: &str, port: u16) -> PyResult<Self> {
        let s = Arc::new(Mutex::new(state));

        let client = Arc::new(RocketClient::new(s.clone(), host, port)?);
        Ok(Self { client, state: s })
    }

    fn connect<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let c = self.client.clone();

        future_into_py(py, async move {
            c.start().await?;
            Ok(())
        })
    }

    fn refuel_oxidizer<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let c = self.client.clone();

        future_into_py(py, async move {
            c.send_servo(SERVO_OXIDIZER_INTAKE, 0).await
        })
    }

    fn close_oxidizer<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let c = self.client.clone();

        future_into_py(py, async move {
            c.send_servo(SERVO_OXIDIZER_INTAKE, 100).await
        })
    }

    fn refuel_fuel<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let c = self.client.clone();

        future_into_py(py, async move {
            c.send_servo(SERVO_FUEL_INTAKE, 0).await
        })
    }

    fn close_fuel<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let c = self.client.clone();

        future_into_py(py, async move {
            c.send_servo(SERVO_FUEL_INTAKE, 100).await
        })
    }

    fn activate_heater<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let c = self.client.clone();

        future_into_py(py, async move {
            c.send_relay(RELAY_HEATER, true).await
        })
    }

    fn deactivate_heater<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let c = self.client.clone();

        future_into_py(py, async move {
            c.send_relay(RELAY_HEATER, false).await
        })
    }

    fn deploy_parachute<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let c = self.client.clone();

        future_into_py(py, async move {
            c.send_relay(RELAY_PARACHUTE, true).await
        })
    }

    fn ignition_sequence<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let c = self.client.clone();
        let s = self.state.clone();

        future_into_py(py, async move {
            let _ = c.send_servo(SERVO_FUEL_MAIN, 0).await;
            let _ = c.send_servo(SERVO_OXIDIZER_MAIN, 0).await;

            tokio::time::sleep(time::Duration::from_millis(150)).await;

            let _ = c.send_relay(RELAY_IGNITER, true).await;

            Python::attach(|py| {
                let st = s.lock();
                let state = st.bind(py);

                state.setattr("step_ignition", true).unwrap();
                state.setattr("flight_stage", "ASCENT").unwrap();
            });

            tokio::time::sleep(time::Duration::from_millis(1000)).await;
            let _ = c.send_relay(RELAY_IGNITER, false).await;

            Ok(())
        })
    }
}
