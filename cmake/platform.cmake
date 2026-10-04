# Include before find_package(Zephyr); application overrides are applied last.
include_guard(GLOBAL)
get_filename_component(HITO_CORE_DIR "${CMAKE_CURRENT_LIST_DIR}/.." ABSOLUTE)

function(hito_core_prepare_zephyr)
  if(NOT DEFINED BOARD)
    set(BOARD nrf5340dk_nrf5340_cpuapp PARENT_SCOPE)
    set(BOARD nrf5340dk_nrf5340_cpuapp)
  endif()
  if(HITO_BOARD_OVERLAY)
    get_filename_component(reference "${HITO_BOARD_OVERLAY}" ABSOLUTE BASE_DIR "${CMAKE_CURRENT_SOURCE_DIR}")
  elseif(BOARD STREQUAL "nrf5340dk_nrf5340_cpuapp")
    set(reference "${HITO_CORE_DIR}/boards/nrf5340dk_nrf5340_cpuapp.overlay")
  else()
    message(FATAL_ERROR "No Hito reference profile for ${BOARD}; set HITO_BOARD_OVERLAY to a complete board overlay")
  endif()
  if(NOT EXISTS "${reference}")
    message(FATAL_ERROR "Hito reference overlay does not exist: ${reference}")
  endif()
  set(overlays "${reference}" ${HITO_OVERLAY_FILES})
  string(REPLACE " " ";" user_overlays "${DTC_OVERLAY_FILE}")
  list(APPEND overlays ${user_overlays})
  list(REMOVE_DUPLICATES overlays)
  set(DTC_OVERLAY_FILE "${overlays}" PARENT_SCOPE)
endfunction()

function(hito_core_add_platform_sources target)
  target_sources(${target} PRIVATE
    "${HITO_CORE_DIR}/main.c"
    "${HITO_CORE_DIR}/shims/zephyr/time.c"
    "${HITO_CORE_DIR}/shims/zephyr/usb_uart.c"
    "${HITO_CORE_DIR}/shims/zephyr/logging.c"
    "${HITO_CORE_DIR}/drivers_c/hito_ble.c"
    "${HITO_CORE_DIR}/drivers_c/ft6336_ctp.c"
    "${HITO_CORE_DIR}/drivers_c/ili9342_lcd.c"
    "${HITO_CORE_DIR}/drivers_c/hito_pin_config.c"
    "${HITO_CORE_DIR}/drivers_c/hito_button.c"
    "${HITO_CORE_DIR}/drivers_c/hito_power.c"
  )
  target_include_directories(${target} PRIVATE
    "${HITO_CORE_DIR}/drivers_c"
    "${HITO_CORE_DIR}/shims/include"
  )
endfunction()

# Link a Cargo staticlib without adding firmware crypto or vault sources.
function(hito_core_add_rust_app target package manifest)
  if(DEFINED ENV{CARGO} AND NOT "$ENV{CARGO}" STREQUAL "")
    set(cargo "$ENV{CARGO}")
  else()
    find_program(cargo cargo REQUIRED)
  endif()
  execute_process(COMMAND "${cargo}" metadata --no-deps --format-version 1 --manifest-path "${manifest}"
    RESULT_VARIABLE result OUTPUT_VARIABLE metadata ERROR_VARIABLE error)
  if(NOT result EQUAL 0)
    message(FATAL_ERROR "cargo metadata failed: ${error}")
  endif()
  string(JSON target_dir GET "${metadata}" target_directory)
  get_filename_component(app_dir "${manifest}" DIRECTORY)
  string(REPLACE "-" "_" library "${package}")
  if(CONFIG_DEBUG)
    set(profile debug)
    set(release)
  else()
    set(profile release)
    set(release --release)
  endif()
  set(archive "${target_dir}/thumbv7em-none-eabi/${profile}/lib${library}.a")
  add_custom_target(${package}_rust
    COMMAND "${cargo}" build --manifest-path "${manifest}" -p "${package}"
      --target thumbv7em-none-eabi ${release} --no-default-features --features zephyr,log-info
    WORKING_DIRECTORY "${app_dir}" BYPRODUCTS "${archive}" USES_TERMINAL)
  add_dependencies(${target} ${package}_rust)
  target_link_libraries(${target} PUBLIC "${archive}")
endfunction()
