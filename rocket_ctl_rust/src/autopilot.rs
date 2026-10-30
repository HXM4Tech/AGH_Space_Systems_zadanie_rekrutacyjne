use std::sync::Arc;

use parking_lot::Mutex;
use pyo3::prelude::*;

use crate::manager::RocketManager;
use crate::state::RocketState;

#[pyclass]
pub struct Autopilot {
    manager: Py<RocketManager>,
    state: Arc<Mutex<Py<RocketState>>>,
    running: Arc<Mutex<bool>>,
}

#[pymethods]
impl Autopilot {
    #[new]
    fn new(manager: Py<RocketManager>) -> PyResult<Self> {
        Python::attach(|py| {
            let m = manager.bind(py);
            let st = m.getattr("state")?;
            let state: Py<RocketState> = st.extract()?;
            Ok(Self {
                manager: manager.clone_ref(py),
                state: Arc::new(Mutex::new(state)),
                running: Arc::new(Mutex::new(false)),
            })
        })
    }

    fn is_running(&self) -> bool {
        *self.running.lock()
    }

    fn stop<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let mut r = self.running.lock();
        *r = false;
        drop(r);

        let mgr = self.manager.clone_ref(py);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            Python::attach(|py| { 
                let _ = mgr.borrow(py).deactivate_heater(py);
            });
            Ok(())
        })
    }

    fn run_mission<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let mut r = self.running.lock();
        if *r {
            return pyo3_async_runtimes::tokio::future_into_py(py, async { Ok(()) });
        }
        *r = true;
        drop(r);

        let manager = self.manager.clone_ref(py);
        let running = self.running.clone();
        let state = self.state.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            // Helper functions to get/set state attributes without explicitly locking the GIL in every call
            let get_f64 = |st: Arc<Mutex<Py<RocketState>>>, attr: &'static str| -> f64 {
                Python::attach(move |py| {
                    let s = st.lock();
                    s.bind(py).getattr(attr).and_then(|v| v.extract::<f64>()).unwrap_or(0.0)
                })
            };

            let get_bool = |st: Arc<Mutex<Py<RocketState>>>, attr: &'static str| -> bool {
                Python::attach(move |py| {
                    let s = st.lock();
                    s.bind(py).getattr(attr).and_then(|v| v.extract::<bool>()).unwrap_or(false)
                })
            };

            let set_status = |st: Arc<Mutex<Py<RocketState>>>, msg: String| {
                Python::attach(move |py| {
                    let s = st.lock();
                    let _ = s.bind(py).setattr("auto_status_text", msg);
                });
            };

            if !*running.lock() {
                let mut r = running.lock();
                *r = false;
                return Ok(());
            }

            if get_f64(state.clone(), "oxidizer_level") < 100.0 {
                set_status(state.clone(), "Refueling oxidizer tank...".to_string());

                Python::attach(|py| { 
                    let _ = manager.borrow(py).refuel_oxidizer(py);
                });

                loop {
                    if !*running.lock() { return Ok(()); }
                    if get_f64(state.clone(), "oxidizer_level") >= 100.0 { break; }
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
            }

            Python::attach(|py| { 
                let _ = manager.borrow(py).close_oxidizer(py);
            });

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            if !*running.lock() {
                let mut r = running.lock();
                *r = false;
                return Ok(());
            }
            
            if get_f64(state.clone(), "fuel_level") < 100.0 {
                set_status(state.clone(), "Refueling fuel tank...".to_string());

                Python::attach(|py| { 
                    let _ = manager.borrow(py).refuel_fuel(py);
                });

                loop {
                    if !*running.lock() { return Ok(()); }
                    if get_f64(state.clone(), "fuel_level") >= 100.0 { break; }
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
            }

            Python::attach(|py| { 
                let _ = manager.borrow(py).close_fuel(py);
            });
            
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            if !*running.lock() {
                let mut r = running.lock();
                *r = false;
                return Ok(());
            }

            set_status(state.clone(), "Heating oxidizer...".to_string());
            
            Python::attach(|py| { 
                let _ = manager.borrow(py).activate_heater(py);
            });
            
            loop {
                if !*running.lock() {
                    Python::attach(|py| { 
                        let _ = manager.borrow(py).deactivate_heater(py);
                    });

                    return Ok(());
                }

                if get_f64(state.clone(), "oxidizer_pressure") >= 59.0 { break; }
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }

            Python::attach(|py| { 
                let _ = manager.borrow(py).deactivate_heater(py);
            });

            if !*running.lock() {
                let mut r = running.lock();
                *r = false;
                return Ok(());
            }

            set_status(state.clone(), "Pressure optimal. Ignition...".to_string());

            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

            if !*running.lock() {
                let mut r = running.lock();
                *r = false;
                return Ok(());
            }

            Python::attach(|py| { 
                let _ = manager.borrow(py).ignition_sequence(py);
            });

            set_status(state.clone(), "Ascent in progress...".to_string());

            loop {
                if !*running.lock() { return Ok(()); }
                if get_bool(state.clone(), "apogee_detected") { break; }

                if get_bool(state.clone(), "step_ignition") && get_f64(state.clone(), "fuel_level") <= 0.1 && get_f64(state.clone(), "max_altitude") > 30.0 {
                    if get_f64(state.clone(), "altitude") < get_f64(state.clone(), "max_altitude") - 1.0 || get_f64(state.clone(), "velocity") <= 0.0 {
                        break;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }

            set_status(state.clone(), "Descent - deploying parachute...".to_string());

            if !*running.lock() { return Ok(()); }
            
            Python::attach(|py| { 
                let _ = manager.borrow(py).deploy_parachute(py);
            });

            set_status(state.clone(), "Landing...".to_string());

            loop {
                if !*running.lock() { return Ok(()); }
                if get_bool(state.clone(), "step_landed") { break; }
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }

            Python::attach(|py| {
                let s = state.lock();
                let _ = s.bind(py).setattr("auto_status_text", py.None());
            });

            let mut r = running.lock();
            *r = false;

            Ok(())
        })
    }
}
