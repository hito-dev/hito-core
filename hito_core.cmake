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

# Main application sources
set(SOURCE_FILES
  ${HITO_CORE_DIR}/main.c
  ${SHIMS_SOURCES}
  ${DRIVERS_C_SOURCES}
)

# Include directories for the C drivers and crypto libs
target_include_directories(app PRIVATE 
  ${ZEPHYR_DRIVERS_DIR}
  ${SHIMS_SOURCE_DIR}/../include
)

target_sources(app PRIVATE ${SOURCE_FILES})
# TODO decide PUBLIC vs PRIVATE
target_link_libraries(app PUBLIC ${RUST_LIB_PATH})

#target_compile_options(app PRIVATE -Os -ffunction-sections -fdata-sections)
target_compile_options(app PRIVATE -Os -ffunction-sections -fdata-sections)
target_link_options(app PUBLIC -Wl,--gc-sections)

