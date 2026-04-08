# hito_core.cmake
set(RUST_TARGET thumbv7em-none-eabi)

# Build configuration variables
if(CONFIG_DEBUG)
    set(RUST_BUILD_TYPE_FLAG "")
    set(RUST_BUILD_TYPE "debug")
else()
    set(RUST_BUILD_TYPE_FLAG "--release")
    set(RUST_BUILD_TYPE "release")
endif()
set(RUST_APP_DIR ${CMAKE_CURRENT_SOURCE_DIR})
set(RUST_WORKSPACE_DIR ${RUST_APP_DIR}/../..)
set(RUST_TARGET_DIR ${RUST_WORKSPACE_DIR}/target/${RUST_TARGET}/${RUST_BUILD_TYPE})
get_filename_component(RUST_LIB_PATH
    ${RUST_TARGET_DIR}/lib${PROJECT_NAME}.a
    ABSOLUTE
)

# Build features and flags
#set(RUST_FEATURES zephyr)

include(${CMAKE_CURRENT_LIST_DIR}/cargo_features.cmake)
set(RUST_FEATURES ${CARGO_FEATURES_STR})

set(RUST_BUILD_FLAGS --target ${RUST_TARGET} ${RUST_BUILD_TYPE_FLAG} --no-default-features --features ${RUST_FEATURES},log-${RUST_LOG_LEVEL} -p ${PROJECT_NAME})

# Custom target to ensure Rust library is built
add_custom_target(rust_build ALL
    COMMAND cargo build ${RUST_BUILD_FLAGS}
    WORKING_DIRECTORY ${RUST_APP_DIR}
    COMMENT "Building Rust library (Cargo handles incremental builds)"
    USES_TERMINAL
    BYPRODUCTS ${RUST_LIB_PATH}
)

# platform shims for zephry inline functions
set(SHIMS_SOURCE_DIR ${HITO_CORE_DIR}/shims/zephyr)
set(SHIMS_SOURCES
  ${SHIMS_SOURCE_DIR}/time.c
  ${SHIMS_SOURCE_DIR}/usb_uart.c
  ${SHIMS_SOURCE_DIR}/logging.c
)

# legacy C hito drivers
set(DRIVERS_C_SOURCE_DIR ${HITO_CORE_DIR}/drivers_c)
set(DRIVERS_C_SOURCES
  ${DRIVERS_C_SOURCE_DIR}/hito_ble.c
  ${DRIVERS_C_SOURCE_DIR}/ft6336_ctp.c
  ${DRIVERS_C_SOURCE_DIR}/ili9342_lcd.c
  ${DRIVERS_C_SOURCE_DIR}/hito_pin_config.c
  ${DRIVERS_C_SOURCE_DIR}/hito_button.c
  ${DRIVERS_C_SOURCE_DIR}/hito_power.c
)

# Add libcrypt0 and related C sources
set(LIBCRYPT0_DIR ${RUST_WORKSPACE_DIR}/crates/libcrypt0-sys)
set(LIBCRYPT0_SOURCES
  ${LIBCRYPT0_DIR}/src/crypt0.c
  ${LIBCRYPT0_DIR}/src/crypt0_aes_ccm.c
  ${LIBCRYPT0_DIR}/src/crypt0_bech32.c
  ${LIBCRYPT0_DIR}/src/crypt0_bip32.c
  ${LIBCRYPT0_DIR}/src/crypt0_bip39.c
  ${LIBCRYPT0_DIR}/src/crypt0_crc.c
  ${LIBCRYPT0_DIR}/src/crypt0_ed25519.c
  ${LIBCRYPT0_DIR}/src/crypt0_hmac.c
  ${LIBCRYPT0_DIR}/src/crypt0_hw_keys.c
  ${LIBCRYPT0_DIR}/src/crypt0_key.c
  ${LIBCRYPT0_DIR}/src/crypt0_log.c
  ${LIBCRYPT0_DIR}/src/crypt0_pbkdf2.c
  ${LIBCRYPT0_DIR}/src/crypt0_ripemd160.c
  ${LIBCRYPT0_DIR}/src/crypt0_rlp.c
  ${LIBCRYPT0_DIR}/src/crypt0_rng.c
  ${LIBCRYPT0_DIR}/src/hito_rand.c
  ${LIBCRYPT0_DIR}/lib/SHA3IUF/sha3.c
  ${LIBCRYPT0_DIR}/src/crypt0_secp256k1.c
  ${LIBCRYPT0_DIR}/src/crypt0_sha.c
  ${LIBCRYPT0_DIR}/src/intc_impl.c
  # ${RUST_SOURCE_DIR}/crypto/crc16_ccitt/crc16_ccitt.c
  # ${RUST_SOURCE_DIR}/vault/vault.c
)

# Main application sources
set(SOURCE_FILES
  ${HITO_CORE_DIR}/main.c
  ${SHIMS_SOURCES}
  ${DRIVERS_C_SOURCES}
)

# Include directories for the C drivers and crypto libs
target_include_directories(app PRIVATE 
  ${DRIVERS_C_SOURCE_DIR}
  ${LIBCRYPT0_DIR}/include
  ${LIBCRYPT0_DIR}/lib/secp256k1/src
  ${LIBCRYPT0_DIR}/lib/secp256k1/include
  ${LIBCRYPT0_DIR}/lib/ripemd160
  ${LIBCRYPT0_DIR}/lib/SHA3IUF
  ${LIBCRYPT0_DIR}/lib/base58
  ${LIBCRYPT0_DIR}/lib/bech32
  ${LIBCRYPT0_DIR}/lib/intc
  ${SHIMS_SOURCE_DIR}/../include
)

target_sources(app PRIVATE ${SOURCE_FILES} ${LIBCRYPT0_SOURCES})
# TODO decide PUBLIC vs PRIVATE
target_link_libraries(app PUBLIC ${RUST_LIB_PATH})

#target_compile_options(app PRIVATE -Os -ffunction-sections -fdata-sections)
target_compile_options(app PRIVATE -Os -ffunction-sections -fdata-sections)
target_link_options(app PUBLIC -Wl,--gc-sections)

