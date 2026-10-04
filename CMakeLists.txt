cmake_minimum_required(VERSION 3.13.1)

# set log level and build type here
set(RUST_LOG_LEVEL trace)

# set application config to point to hito_core (prj.conf and boards overlays)
set(HITO_CORE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../crates/hito_core)
set(APPLICATION_CONFIG_DIR ${HITO_CORE_DIR})
find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})

# set project name
get_filename_component(PROJECT_DIR_NAME ${CMAKE_CURRENT_SOURCE_DIR} NAME)
project(${PROJECT_DIR_NAME})

# include hito_core cmake (zephyr and rust build logic)
include(${HITO_CORE_DIR}/hito_core.cmake)
