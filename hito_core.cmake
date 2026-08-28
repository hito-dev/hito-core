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
string(REPLACE "-" "_" RUST_CRATE_LIB_NAME "${PROJECT_NAME}")
get_filename_component(RUST_LIB_PATH
    ${RUST_TARGET_DIR}/lib${RUST_CRATE_LIB_NAME}.a
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
  # ${SHIMS_SOURCE_DIR}/vault.c
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
  ${LIBCRYPT0_DIR}/src/src_c/crypt0.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_aes_ccm.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_bech32.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_bip32.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_bip39.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_crc.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_ed25519.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_hmac.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_hw_keys.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_key.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_log.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_pbkdf2.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_ripemd160.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_rlp.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_rng.c
  ${LIBCRYPT0_DIR}/src/src_c/hito_rand.c
  ${LIBCRYPT0_DIR}/lib/SHA3IUF/sha3.c
  ${LIBCRYPT0_DIR}/lib/secp256k1/src/secp256k1.c
  ${LIBCRYPT0_DIR}/lib/base58/base58.c
  # ${LIBCRYPT0_DIR}/lib/secp256k1/src/precomputed_ecmult.c
  ${LIBCRYPT0_DIR}/lib/secp256k1/src/precomputed_ecmult_gen.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_secp256k1.c
  ${LIBCRYPT0_DIR}/src/src_c/crypt0_sha.c
  ${LIBCRYPT0_DIR}/src/src_c/intc_impl.c
  # ${RUST_SOURCE_DIR}/crypto/crc16_ccitt/crc16_ccitt.c
  # ${RUST_SOURCE_DIR}/vault/vault.c
)

# The legacy C Vault is selected explicitly by the application's Zephyr
# feature. Keep its sources out of unrelated embedded applications.
file(READ ${RUST_APP_DIR}/Cargo.toml RUST_APP_CARGO_MANIFEST)
string(FIND "${RUST_APP_CARGO_MANIFEST}" "hito-vault/c-vault-backend" HITO_C_VAULT_FEATURE_POS)
set(HITO_VAULT_C_SOURCES "")
if(NOT HITO_C_VAULT_FEATURE_POS EQUAL -1)
  set(HITO_VAULT_C_DIR ${RUST_WORKSPACE_DIR}/crates/hito-vault/legacy-c)
  set(CRYPT0PRO_C_DIR ${RUST_WORKSPACE_DIR}/crates/crypt0x/legacy-c)
  list(APPEND HITO_VAULT_C_SOURCES
    ${HITO_VAULT_C_DIR}/hito_vault.c
    ${HITO_VAULT_C_DIR}/hito_vault_ffi.c
    ${HITO_VAULT_C_DIR}/hito_vault_platform.c
    ${HITO_VAULT_C_DIR}/hito_boot_version.c
    ${CRYPT0PRO_C_DIR}/src/crypt0pro_eth.c
    ${CRYPT0PRO_C_DIR}/src/crypt0pro_near.c
    ${CRYPT0PRO_C_DIR}/src/crypt0pro_solana.c
  )
endif()

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

if(NOT HITO_C_VAULT_FEATURE_POS EQUAL -1)
  target_include_directories(app PRIVATE
    ${HITO_VAULT_C_DIR}
    ${CRYPT0PRO_C_DIR}/include
  )
endif()

target_sources(app PRIVATE ${SOURCE_FILES} ${LIBCRYPT0_SOURCES} ${HITO_VAULT_C_SOURCES})
target_compile_definitions(app PRIVATE
  # ENABLE_MODULE_ECDH
  # ENABLE_MODULE_RECOVERY
  # USE_FORCE_WIDEMUL_INT64
  ECMULT_GEN_PREC_BITS=2
)
# TODO decide PUBLIC vs PRIVATE
target_link_libraries(app PUBLIC ${RUST_LIB_PATH})

#target_compile_options(app PRIVATE -Os -ffunction-sections -fdata-sections)
target_compile_options(app PRIVATE -Os -ffunction-sections -fdata-sections)
target_link_options(app PUBLIC -Wl,--gc-sections)
