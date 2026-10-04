# Extraction validation

Source tree and commit metadata: 73 files and 108 commits verified.

On macOS with Rust stable:
- `cargo test -p hito_core`: 29 tests passed.
- `cargo build --locked -p hito_core_devboard`: passed.
- `cargo check --locked -p hito_core --target thumbv7em-none-eabi --no-default-features --features zephyr,log-info`: passed.

With Nordic nRF Connect SDK 1.9.1:
- `west build -p always -b nrf5340dk_nrf5340_cpuapp examples/devboard`: passed, including the Bluetooth network-core image. Main image: 93,204 bytes flash and 249,245 bytes SRAM.

The devboard image was built but has not been flashed or tested on attached LCD/touch hardware. Existing native-driver warnings remain.
