# cargo-features.cmake
# Reads default features from Cargo.toml via `cargo metadata` and builds
# a feature string for use with --no-default-features --features <...>.
#
# Usage:
#   include(${CMAKE_CURRENT_LIST_DIR}/cargo-features.cmake)
#   # Then use CARGO_FEATURES_STR in your build command, e.g.:
#   #   cargo build --no-default-features --features ${CARGO_FEATURES_STR}
#
# Always-on features (added regardless of Cargo.toml defaults):
#   set(CARGO_ALWAYS_ON_FEATURES "zephyr")
#
# Optional features gated by Cargo.toml defaults (extend as needed):
#   set(CARGO_OPTIONAL_FEATURES "gui" "audio" "bluetooth")

# --- Configuration --------------------------------------------------------

# Features always included in the build
set(CARGO_ALWAYS_ON_FEATURES "zephyr")

# Features that will be included only if listed in Cargo.toml [features] default
set(CARGO_OPTIONAL_FEATURES "gui" "audio" "bluetooth")

# ---------------------------------------------------------------------------

message(STATUS "Analyzing Cargo.toml features...")

execute_process(
  COMMAND cargo metadata --no-deps --format-version 1
  WORKING_DIRECTORY ${CMAKE_CURRENT_LIST_DIR}
  OUTPUT_VARIABLE CARGO_METADATA
  RESULT_VARIABLE CARGO_METADATA_RESULT
  OUTPUT_STRIP_TRAILING_WHITESPACE
)

if(NOT CARGO_METADATA_RESULT EQUAL 0)
  message(FATAL_ERROR "cargo metadata failed. Is cargo installed and is there a Cargo.toml in ${CMAKE_CURRENT_LIST_DIR}?")
endif()

# Find the package by matching CMake project name to Cargo package name
string(JSON PKG_COUNT LENGTH "${CARGO_METADATA}" "packages")
math(EXPR PKG_LAST "${PKG_COUNT} - 1")

set(CARGO_DEFAULT_FEATURES "")

foreach(i RANGE ${PKG_LAST})
  string(JSON PKG_NAME GET "${CARGO_METADATA}" "packages" ${i} "name")
  if(PKG_NAME STREQUAL PROJECT_NAME)
    string(JSON CARGO_DEFAULT_FEATURES GET "${CARGO_METADATA}" "packages" ${i} "features" "default")
    message(STATUS "Found crate: ${PKG_NAME}, default features: ${CARGO_DEFAULT_FEATURES}")
    break()
  endif()
endforeach()

# Start with always-on features
set(CARGO_FEATURE_LIST ${CARGO_ALWAYS_ON_FEATURES})

# Append optional features that appear in Cargo.toml defaults
foreach(FEATURE ${CARGO_OPTIONAL_FEATURES})
  if(CARGO_DEFAULT_FEATURES MATCHES "\"${FEATURE}\"")
    list(APPEND CARGO_FEATURE_LIST "${FEATURE}")
    message(STATUS "  [cargo] Enabling optional feature from defaults: ${FEATURE}")
  endif()
endforeach()

list(JOIN CARGO_FEATURE_LIST "," CARGO_FEATURES_STR)
message(STATUS "Cargo features resolved: ${CARGO_FEATURES_STR}")
