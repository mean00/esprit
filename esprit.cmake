#=============================================================================#
# Author: Tomasz Bogdal (QueezyTheGreat)
# Home:   https://github.com/queezythegreat/arduino-cmake
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.
#=============================================================================#
message(STATUS "Invoking esprit.cmake for MCU=${LN_MCU}")
OPTION(USE_SCAN_BUILD "Disable custom CC")
IF(NOT DEFINED LN_ARCH)
  MESSAGE(STATUS "Architecture not defined, reverting to RISCV")
  SET(LN_ARCH "RISCV" CACHE INTERNAL "")
ENDIF()

STRING(TOUPPER ${LN_ARCH} LN_ARCH)
SET(LN_MCU_TOOLCHAIN_PREFIX "UNKNOWN")
IF(LN_ARCH STREQUAL "RISCV")
  IF("${LN_MCU}" STREQUAL "CH32V3x")
    SET(LN_MCU_TOOLCHAIN_PREFIX ${CMAKE_CURRENT_LIST_DIR}/mcus/riscv_ch32v3x/ CACHE INTERNAL "")
  ELSEIF("${LN_MCU}" STREQUAL "RP2350_RISCV")
    SET(LN_MCU_TOOLCHAIN_PREFIX ${CMAKE_CURRENT_LIST_DIR}/mcus/arm_rp2350/riscv/ CACHE INTERNAL "")
  ELSEIF("${LN_MCU}" STREQUAL "ESP32")
    SET(LN_MCU_TOOLCHAIN_PREFIX ${CMAKE_CURRENT_LIST_DIR}/mcus/riscv_esp32/ CACHE INTERNAL "")
  ENDIF()
ELSEIF(LN_ARCH STREQUAL "ARM")
  IF("${LN_MCU}" STREQUAL "RP2040")
    SET(LN_MCU_TOOLCHAIN_PREFIX ${CMAKE_CURRENT_LIST_DIR}/mcus/arm_rp2040/ CACHE INTERNAL "")
  ELSEIF("${LN_MCU}" STREQUAL "RP2350")
    SET(LN_MCU_TOOLCHAIN_PREFIX ${CMAKE_CURRENT_LIST_DIR}/mcus/arm_rp2350/ CACHE INTERNAL "")
  ELSE()
    SET(LN_MCU_TOOLCHAIN_PREFIX ${CMAKE_CURRENT_LIST_DIR}/mcus/arm_gd32fx/ CACHE INTERNAL "")
  ENDIF()
ELSE()
  MESSAGE(FATAL_ERROR "LN_ARCH UNSUPPORTED, SET IT TO EITHER RISCV or ARM (LN_MCU=${LN_ARCH})")
ENDIF()

IF("${LN_MCU_TOOLCHAIN_PREFIX}" STREQUAL "UNKNOWN")
  MESSAGE(FATAL_ERROR "LN_MCU_TOOLCHAIN_PREFIX UNSUPPORTED (LN_MCU=${LN_MCU}, LN_ARCH=${LN_ARCH})")
ENDIF()

SET(LN_MCU_TOOLCHAIN_PREFIX ${LN_MCU_TOOLCHAIN_PREFIX} CACHE INTERNAL "")

IF(USE_CLANG)
  include(${LN_MCU_TOOLCHAIN_PREFIX}/toolchain_clang.cmake)
ELSE()
  include(${LN_MCU_TOOLCHAIN_PREFIX}/toolchain.cmake)
ENDIF()

SET(CMAKE_C_COMPILER_FORCED TRUE CACHE INTERNAL "")
SET(CMAKE_CXX_COMPILER_FORCED TRUE CACHE INTERNAL "")

message(STATUS "// Invokation esprit.cmake done for MCU=${LN_MCU}")

