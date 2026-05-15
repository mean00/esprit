# =============================================================================#
# Author: Tomasz Bogdal (QueezyTheGreat) Home:   https://github.com/queezythegreat/arduino-cmake
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL was not
# distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/.
# =============================================================================#
message(STATUS "Invoking esprit.cmake for MCU=${LN_MCU}")
option(USE_SCAN_BUILD "Disable custom CC")
if(NOT DEFINED LN_ARCH)
  message(STATUS "Architecture not defined, reverting to RISCV")
  set(LN_ARCH
      "RISCV"
      CACHE INTERNAL "")
endif()

string(TOUPPER ${LN_ARCH} LN_ARCH)
set(LN_MCU_TOOLCHAIN_PREFIX "UNKNOWN")
if(LN_ARCH STREQUAL "RISCV")
  if("${LN_MCU}" STREQUAL "CH32V3x")
    set(LN_MCU_TOOLCHAIN_PREFIX
        ${CMAKE_CURRENT_LIST_DIR}/mcus/riscv_ch32v3x/
        CACHE INTERNAL "")
  elseif("${LN_MCU}" STREQUAL "RP2350_RISCV")
    set(LN_MCU_TOOLCHAIN_PREFIX
        ${CMAKE_CURRENT_LIST_DIR}/mcus/arm_rp2350/riscv/
        CACHE INTERNAL "")
  elseif("${LN_MCU}" STREQUAL "ESP32")
    set(LN_MCU_TOOLCHAIN_PREFIX
        ${CMAKE_CURRENT_LIST_DIR}/mcus/riscv_esp32/
        CACHE INTERNAL "")
  endif()
elseif(LN_ARCH STREQUAL "ARM")
  if("${LN_MCU}" STREQUAL "RP2040")
    set(LN_MCU_TOOLCHAIN_PREFIX
        ${CMAKE_CURRENT_LIST_DIR}/mcus/arm_rp2040/
        CACHE INTERNAL "")
  elseif("${LN_MCU}" STREQUAL "RP2350")
    set(LN_MCU_TOOLCHAIN_PREFIX
        ${CMAKE_CURRENT_LIST_DIR}/mcus/arm_rp2350/
        CACHE INTERNAL "")
  else()
    set(LN_MCU_TOOLCHAIN_PREFIX
        ${CMAKE_CURRENT_LIST_DIR}/mcus/arm_gd32fx/
        CACHE INTERNAL "")
  endif()
else()
  message(FATAL_ERROR "LN_ARCH UNSUPPORTED, SET IT TO EITHER RISCV or ARM (LN_MCU=${LN_ARCH})")
endif()

if("${LN_MCU_TOOLCHAIN_PREFIX}" STREQUAL "UNKNOWN")
  message(FATAL_ERROR "LN_MCU_TOOLCHAIN_PREFIX UNSUPPORTED (LN_MCU=${LN_MCU}, LN_ARCH=${LN_ARCH})")
endif()

set(LN_MCU_TOOLCHAIN_PREFIX
    ${LN_MCU_TOOLCHAIN_PREFIX}
    CACHE INTERNAL "")

if(USE_CLANG)
  include(${LN_MCU_TOOLCHAIN_PREFIX}/toolchain_clang.cmake)
else()
  include(${LN_MCU_TOOLCHAIN_PREFIX}/toolchain.cmake)
endif()

set(CMAKE_C_COMPILER_FORCED
    TRUE
    CACHE INTERNAL "")
set(CMAKE_CXX_COMPILER_FORCED
    TRUE
    CACHE INTERNAL "")

message(STATUS "// Invokation esprit.cmake done for MCU=${LN_MCU}")
