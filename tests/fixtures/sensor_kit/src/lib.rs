pub struct Sensor {
    pub id: u32,
    pub address: u8,
    pub scale: f32,
    pub offset: f32,
    pub connected: bool,
}

// This function reads the sensor. We take a lot of arguments because
// the sensor needs all of them. The bool at the end is whether to
// retry, which we need because the hardware is flaky.
pub fn read_sensor(
    id: u32,
    address: u8,
    scale: f32,
    offset: f32,
    retry: bool,
    power_cycle: bool,
    verbose: bool,
    log_errors: bool,
    timeout_ms: u32,
) -> Result<f32, String> {
    if timeout_ms == 0 {
        return Err("bad timeout".to_string());
    }

    if power_cycle {
        // Pulse the reset line before sampling; the sensor needs a moment
        // to come back up after a power cycle.
        reset_bus(address);
    }

    let mut attempts_left = if retry { 3 } else { 1 };
    loop {
        match sample_raw(address) {
            Ok(raw) => {
                let value = (f32::from(raw) * scale) + offset + id as f32;
                if verbose {
                    println!("sensor {id} @ 0x{address:02x} -> {value}");
                }
                return Ok(value);
            }
            Err(err) => {
                attempts_left -= 1;
                if attempts_left == 0 {
                    if log_errors {
                        eprintln!("read_sensor giving up on sensor {id}: {err}");
                    }
                    return Err(err);
                }
            }
        }
    }
}

fn reset_bus(_address: u8) {}

fn sample_raw(address: u8) -> Result<u8, String> {
    if address == 0 {
        // 0x00 is the I2C general call address; no sensor answers on it.
        return Err("no device at address 0x00".to_string());
    }
    Ok(address)
}

pub fn connect(s: Sensor) -> Result<Sensor, Box<dyn std::error::Error>> {
    // We have to take ownership here because of borrow checker issues.
    let mut s = s;
    s.connected = true;
    Ok(s)
}

pub fn calibrate(s: &Sensor, raw: f32) -> f32 {
    raw * s.scale + s.offset
}
