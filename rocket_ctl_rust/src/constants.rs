pub const BOARD_SOFTWARE: u8 = 0x01;
pub const BOARD_ROCKET: u8 = 0x02;

pub const ACTION_FEED: u8 = 0x00;
pub const ACTION_SERVICE: u8 = 0x01;
pub const ACTION_ACK: u8 = 0x02;
pub const ACTION_NACK: u8 = 0x03;

pub const PRIORITY_HIGH: u8 = 0x00;
pub const PRIORITY_LOW: u8 = 0x01;

pub const DEVICE_SERVO: u8 = 0x00;
pub const DEVICE_RELAY: u8 = 0x01;
pub const DEVICE_SENSOR: u8 = 0x02;

pub const SERVO_FUEL_INTAKE: u8 = 0x00;
pub const SERVO_OXIDIZER_INTAKE: u8 = 0x01;
pub const SERVO_FUEL_MAIN: u8 = 0x02;
pub const SERVO_OXIDIZER_MAIN: u8 = 0x03;

pub const RELAY_HEATER: u8 = 0x00;
pub const RELAY_IGNITER: u8 = 0x01;
pub const RELAY_PARACHUTE: u8 = 0x02;
