import asyncio
import logging
from typing import Optional

import rocket_comm_rust
from state import RocketState

logger = logging.getLogger("RocketClient")

class RocketClient:
    def __init__(self, state: RocketState, host: str = "127.0.0.1", port: int = 3000):
        self.state = state
        self.host = host
        self.port = port
        self.reader: Optional[asyncio.StreamReader] = None
        self.writer: Optional[asyncio.StreamWriter] = None
        self.is_running: bool = False
        self._manager_task: Optional[asyncio.Task] = None

    def start(self):
        """Starts the background connection manager task."""
        if not self.is_running:
            self.is_running = True
            try:
                self._manager_task = asyncio.create_task(self._connection_manager())
            except RuntimeError:
                pass

    async def _connection_manager(self):
        """Resilient loop keeping the connection alive with the proxy."""
        while self.is_running:
            if not self.state.connected:
                try:
                    self.reader, self.writer = await asyncio.open_connection(
                        self.host, self.port
                    )
                    self.state.connected = True
                    logger.info(f"Connected to TCP Proxy at {self.host}:{self.port}")
                    await self._listen_loop()
                except (ConnectionRefusedError, OSError):
                    self.state.connected = False
                    await asyncio.sleep(1.0)
                except Exception as e:
                    logger.error(f"Unexpected connection error: {e}")
                    self.state.connected = False
                    await asyncio.sleep(1.0)
            else:
                await asyncio.sleep(0.5)

    async def _listen_loop(self):
        """Reads and dispatches frames from TCP stream."""
        while self.is_running and self.state.connected and self.reader:
            try:
                frame = rocket_comm_rust.parse_frame(await self.reader.readexactly(14))
                self.state.update_from_telemetry(frame)
            except (asyncio.IncompleteReadError, ConnectionResetError, BrokenPipeError):
                logger.warning("TCP socket disconnected by peer")
                self.state.connected = False
                break
            except Exception as e:
                logger.error(f"Error parsing frame: {e}")

    async def send_servo(self, device_id: int, position: int) -> bool:
        if self.writer and self.state.connected:
            try:
                data = rocket_comm_rust.build_servo_command(device_id, position)
                self.writer.write(bytes(data))
                await self.writer.drain()
                self.state.servos[device_id] = position
                return True
            except Exception as e:
                logger.error(f"Failed to send servo command: {e}")
                self.state.connected = False
        return False

    async def send_relay(self, device_id: int, open_state: bool) -> bool:
        if self.writer and self.state.connected:
            try:
                data = rocket_comm_rust.build_relay_command(device_id, open_state)
                self.writer.write(bytes(data))
                await self.writer.drain()
                self.state.relays[device_id] = open_state
                if device_id == 2 and open_state:  # RELAY_PARACHUTE
                    self.state.parachute_deployed = True
                    self.state.step_parachute_deployed = True
                    self.state.flight_stage = "DESCENT (PARACHUTE OPEN)"
                return True
            except Exception as e:
                logger.error(f"Failed to send relay command: {e}")
                self.state.connected = False
        return False
