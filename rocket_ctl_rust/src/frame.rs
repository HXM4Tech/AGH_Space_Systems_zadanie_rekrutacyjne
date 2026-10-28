use pyo3::prelude::*;

use crate::checksum::calculate_crc32;
use crate::constants::*;

#[pyclass(get_all)]
#[derive(Clone, Debug, PartialEq)]
pub struct TelemetryFrame {
    pub destination: u8,
    pub priority: u8,
    pub action: u8,
    pub source: u8,
    pub device_type: u8,
    pub device_id: u8,
    pub data_type: u8,
    pub operation: u8,
    pub value_float: Option<f32>,
    pub value_int: Option<i32>,
}

#[pymethods]
impl TelemetryFrame {
    fn __repr__(&self) -> String {
        format!(
            "TelemetryFrame(
                destination={},
                priority={},
                source={},
                device_type={},
                device_id={},
                data_type={},
                operation={},
                value_float={:?},
                value_int={:?}
            )",
            self.destination, self.priority, self.source,
            self.device_type, self.device_id, self.data_type,
            self.operation, self.value_float, self.value_int
        )
    }
}

pub fn parse_raw_frame(data: &[u8]) -> Result<TelemetryFrame, String> {
    if data.len() != 14 {
        return Err(format!("Invalid length: expected 14, got {}", data.len()));
    }
    if data[0] != 0x05 {
        return Err(format!("Invalid magic number: expected 0x05, got 0x{:02X}", data[0]));
    }

    let expected_crc = &data[10..14];
    let calculated_crc = calculate_crc32(&data[0..10]);

    if expected_crc != calculated_crc {
        return Err("Checksum mismatch".to_string());
    }

    let mut buf = [0u8; 8];
    buf[..5].copy_from_slice(&data[1..6]);
    let val = u64::from_le_bytes(buf);

    let destination = (val & 0x1F) as u8;
    let priority    = ((val >> 5) & 0x03) as u8;
    let action      = ((val >> 7) & 0x0F) as u8;
    let source      = ((val >> 11) & 0x1F) as u8;
    let device_type = ((val >> 16) & 0x3F) as u8;
    let device_id   = ((val >> 22) & 0x3F) as u8;
    let data_type   = ((val >> 28) & 0x0F) as u8;
    let operation   = ((val >> 32) & 0xFF) as u8;

    let mut value_float = None;
    let mut value_int = None;

    match data_type {
        0x07 => {
            // FLOAT (f32)
            let payload_bytes: [u8; 4] = data[6..10].try_into().unwrap();
            value_float = Some(f32::from_le_bytes(payload_bytes));
        }
        0x05 => {
            // INT16
            let payload_bytes: [u8; 2] = data[6..8].try_into().unwrap();
            value_int = Some(i16::from_le_bytes(payload_bytes) as i32);
        }
        0x01 => {
            // UINT32
            let payload_bytes: [u8; 4] = data[6..10].try_into().unwrap();
            value_int = Some(u32::from_le_bytes(payload_bytes) as i32);
        }
        0x04 => {
            // INT32
            let payload_bytes: [u8; 4] = data[6..10].try_into().unwrap();
            value_int = Some(i32::from_le_bytes(payload_bytes));
        }
        _ => {}
    }

    Ok(TelemetryFrame {
        destination, priority, action, source,
        device_type, device_id, data_type, operation,
        value_float, value_int,
    })
}

pub fn build_command_frame(
    destination: u8,
    device_type: u8,
    device_id: u8,
    operation: u8,
    data_type: u8,
    payload: [u8; 4],
) -> Vec<u8> {
    build_full_frame(
        destination,
        PRIORITY_LOW,
        ACTION_SERVICE,
        BOARD_SOFTWARE,
        device_type,
        device_id,
        data_type,
        operation,
        payload,
    )
}

pub fn build_full_frame(
    destination: u8,
    priority: u8,
    action: u8,
    source: u8,
    device_type: u8,
    device_id: u8,
    data_type: u8,
    operation: u8,
    payload: [u8; 4],
) -> Vec<u8> {
    let mut out = vec![0u8; 14];
    out[0] = 0x05;

    let val: u64 = (destination as u64 & 0x1F)
        | ((priority as u64 & 0x03) << 5)
        | ((action as u64 & 0x0F) << 7)
        | ((source as u64 & 0x1F) << 11)
        | ((device_type as u64 & 0x3F) << 16)
        | ((device_id as u64 & 0x3F) << 22)
        | ((data_type as u64 & 0x0F) << 28)
        | ((operation as u64 & 0xFF) << 32);

    let header_bytes = val.to_le_bytes();
    out[1..6].copy_from_slice(&header_bytes[..5]);
    out[6..10].copy_from_slice(&payload);

    let crc = calculate_crc32(&out[..10]);
    out[10..14].copy_from_slice(&crc);

    out
}
