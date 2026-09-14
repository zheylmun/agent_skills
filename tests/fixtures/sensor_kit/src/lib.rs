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
    if !retry && timeout_ms == 0 {
        return Err("bad timeout".to_string());
    }
    if power_cycle && verbose {
        return Err("power cycle required".to_string());
    }
    if log_errors && timeout_ms == 0 {
        return Err("timeout with logging enabled".to_string());
    }
    Ok((f32::from(address) * scale) + offset + id as f32)
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
