use std::sync::Arc;
use parking_lot::Mutex;

use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::runtime::Builder;
use tokio::sync::Mutex as TokioMutex;

use crate::frame::{build_command_frame, BOARD_ROCKET, DEVICE_RELAY, DEVICE_SERVO};

#[pyclass]
pub struct RocketClient {
    state: Arc<Mutex<Py<PyAny>>>,
    host: String,
    port: u16,
    writer: Arc<TokioMutex<Option<tokio::net::tcp::OwnedWriteHalf>>>,
    running: Arc<Mutex<bool>>,
}

#[pymethods]
impl RocketClient {
    #[new]
    #[pyo3(signature = (state, host="127.0.0.1", port=3000))]
    fn new(state: Py<PyAny>, host: &str, port: u16) -> PyResult<Self> {
        Ok(Self {
            state: Arc::new(Mutex::new(state)),
            host: host.to_string(),
            port,
            writer: Arc::new(TokioMutex::new(None)),
            running: Arc::new(Mutex::new(false)),
        })
    }

    fn start(&self) -> PyResult<()> {
        {
            let mut running = self.running.lock();
            if *running {
                return Ok(());
            }
            *running = true;
        }

        let state = self.state.clone();
        let writer = self.writer.clone();
        let running = self.running.clone();
        let host = self.host.clone();
        let port = self.port;

        std::thread::spawn(move || {
            let rt = match Builder::new_current_thread().enable_all().build() {
                Ok(r) => r,
                Err(_) => return,
            };

            rt.block_on(async move {
                loop {
                    if !*running.lock() { break };

                    let should_connect = Python::attach(|py| {
                        let st = state.lock();
                        match st.bind(py).getattr("connected") {
                            Ok(c) => c.extract::<bool>().unwrap_or(false) == false,
                            Err(_) => true,
                        }
                    });

                    if !should_connect {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        continue;
                    }

                    match TcpStream::connect((host.as_str(), port)).await {
                        Ok(stream) => {
                            let (mut reader, write_half) = stream.into_split();
                            Python::attach(|py| {
                                let st = state.lock();
                                let _ = st.bind(py).setattr("connected", true);
                            });

                            let mut w = writer.lock().await;
                            *w = Some(write_half);
                            drop(w);

                            let mut buf = [0u8; 14];
                            loop {
                                match reader.read_exact(&mut buf).await {
                                    Ok(_) => {
                                        match crate::frame::parse_raw_frame(&buf) {
                                            Ok(frame) => {
                                                Python::attach(|py| {
                                                    let st = state.lock();
                                                    let _ = st.bind(py).call_method1("update_from_telemetry", (frame,));
                                                });
                                            }
                                            Err(e) => {
                                                eprintln!("Error parsing frame: {e}");
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        Python::attach(|py| {
                                            let st = state.lock();
                                            let _ = st.bind(py).setattr("connected", false);
                                        });

                                        let mut w = writer.lock().await;
                                        *w = None;
                                        drop(w);
                                        
                                        break;
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            Python::attach(|py| {
                                let st = state.lock();
                                let _ = st.bind(py).setattr("connected", false);
                            });
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }
                    }
                }
            });
        });
        Ok(())
    }

    fn send_servo<'py>(&self, py: Python<'py>, device_id: u8, position: i16) -> PyResult<Bound<'py, PyAny>> {
        let writer = self.writer.clone();
        let state = self.state.clone();

        future_into_py(py, async move {
            let data = build_command_frame(BOARD_ROCKET, DEVICE_SERVO, device_id, 5, 5, [position as u8, (position >> 8) as u8, 0, 0]);
            let mut wrote = false;

            let mut w = writer.lock().await;
            if let Some(writer) = w.as_mut() {
                if writer.write_all(&data).await.is_ok() {
                    if writer.flush().await.is_ok() {
                        wrote = true;
                    }
                }
            }
            drop(w);

            if wrote {
                Python::attach(|py| {
                    let st = state.lock();
                    if let Ok(servos) = st.bind(py).getattr("servos") {
                        let _ = servos.set_item(device_id, position as i32);
                    }
                });
            }
            Ok(wrote)
        })
    }

    fn send_relay<'py>(&self, py: Python<'py>, device_id: u8, open_state: bool) -> PyResult<Bound<'py, PyAny>> {
        let writer = self.writer.clone();
        let state = self.state.clone();

        future_into_py(py, async move {
            let op = if open_state { 1u8 } else { 2u8 };
            let data = build_command_frame(BOARD_ROCKET, DEVICE_RELAY, device_id, op, 0, [0u8; 4]);
            let mut wrote = false;
            
            let mut w = writer.lock().await;
            if let Some(writer) = w.as_mut() {
                if writer.write_all(&data).await.is_ok() {
                    if writer.flush().await.is_ok() {
                        wrote = true;
                    }
                }
            }
            drop(w);
            
            if wrote {
                Python::attach(|py| {
                    let st = state.lock();
                    if let Ok(relays) = st.bind(py).getattr("relays") {
                        let _ = relays.set_item(device_id, open_state);
                    }
                    if device_id == 2 && open_state {
                        let _ = st.bind(py).setattr("parachute_deployed", true);
                        let _ = st.bind(py).setattr("step_parachute_deployed", true);
                        let _ = st.bind(py).setattr("flight_stage", "DESCENT (PARACHUTE OPEN)");
                    }
                });
            }
            Ok(wrote)
        })
    }
}
