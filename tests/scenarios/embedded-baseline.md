# Scenario: embedded-baseline

Add a function to this `#![no_std]` driver that reads a burst of samples from the device and returns them to the caller.

```rust
#![no_std]
pub struct Device;
impl Device {
    pub fn read_one(&mut self) -> u16 { 0 }
}
```
