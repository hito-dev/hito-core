# Hito Core

Platform library for Hito devices, with hardware drivers and a desktop simulator.
It provides display and touch access, timing, logging, allocation, and transport.
The same Rust APIs run on the desktop or Nordic Zephyr targets.

## Desktop example

Install Rust, then run `cargo run -p hito_core_devboard`. The simulator displays
a green bar; touching/clicking the display draws white pixels.

## Devboard example

The reference profile targets `nrf5340dk_nrf5340_cpuapp` with Nordic NCS 1.9.1.
Install that SDK, Rust's `thumbv7em-none-eabi` target, `west`, and CMake 3.19+.
Use the SDK's Zephyr/toolchain environment (`build_env.zsh` documents the current
local installation), then build from this repository:

```sh
west build -b nrf5340dk_nrf5340_cpuapp examples/devboard -d build/devboard
west flash -d build/devboard
```

Connect an ILI9342 display and FT6336-compatible touch panel using the reference
overlay. Pins use Nordic's encoded GPIO numbering: P0.n = n, P1.n = 32 + n.

| Connection | Reference pin |
| --- | --- |
| LCD SCK / MOSI / CS | P1.04 / P0.24 / P0.19 |
| LCD DC / reset / power / backlight | P1.08 / P0.21 / P1.06 / P0.16 |
| Touch SDA / SCL | P0.07 / P0.25 |
| Touch interrupt / reset / power | P0.27 / P0.26 / P1.10 |
| Power button | P0.23 (DK Button1) |
| RGB / debug LEDs | P0.28 / P0.29 / P0.30 / P0.31 |
| Devboard detection | P0.03 connected to VDD |

The overlay also defines the existing NAND and charging pins. The display/touch
peripherals are required for their corresponding functions; a bare DK does not
contain them. Check peripheral supply requirements before wiring them.

## Application overrides

Include `cmake/platform.cmake` before `find_package(Zephyr)` and call
`hito_core_prepare_zephyr()`. The reference overlay is applied first; overlays in
`HITO_OVERLAY_FILES` and explicit `DTC_OVERLAY_FILE` inputs follow it.
For example, an application can override LEDs in its own overlay:

```dts
/ {
  zephyr,user {
    led-red-pin = <4>;
    led-green-pin = <6>;
    led-blue-pin = <5>;
  };
};
```

Keep related bus and GPIO definitions consistent when changing LCD CS or PWM
backlight wiring. The aliases `hito-lcd` and `hito-ctp`, and the pin properties in
`zephyr,user`, are the driver configuration interface. Unknown boards require
an explicit complete `HITO_BOARD_OVERLAY`.

The reference profile reserves the last 2 KiB of the current SRAM layout.
Brightness persistence uses its last two bytes at fixed addresses. Applications
must preserve this reservation unless they also adapt the retention drivers.
Firmware flash partitions, watchdog settings, product identity, and crypto/vault
configuration belong to the consuming application.

`hito_core_add_platform_sources(app)` adds the native drivers and shims.
`hito_core_add_rust_app(app package Cargo.toml)` links an application's Rust
static library. `cmake/resolve-core.cmake` finds a Git/path Core dependency using
Cargo metadata, including development overrides.

## History and license

This repository preserves 108 Core commits from `hito-dev/hito-firmware` through
`927a5b6`. See `docs/provenance/README.md` for the source snapshot, mapping,
verification, and extraction recipe. The GitHub initialization commits are also
preserved. Extraction and packaging changes appear afterward.

MIT for original code; see `THIRD_PARTY_NOTICES.md` for retained exceptions.
