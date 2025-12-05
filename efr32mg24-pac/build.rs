// PAC crates should not provide memory.x
// The memory layout is device-specific and should be provided by the HAL or application
fn main() {
    // This build script is intentionally empty
    // memory.x should be provided by efr32mg24-hal or the application
}
