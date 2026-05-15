# Select platform and installation for extra arduino files AND toolchain (compiler...)
set(PLATFORM_ARCHITECTURE "STM32F1")
# Must be named arduino else the cmake code will look for ARDUINO while the name will be the name of the folder
set(PLATFORM_PATH "/home/fx/Arduino/hardware/arduino/")
# SET(PLATFORM_TOOLCHAIN_PATH "/home/fx/.arduino15/./packages/arduino/tools/arm-none-eabi-gcc/4.8.3-2014q1/bin")
# SET(PLATFORM_TOOLCHAIN_PATH "/home/fx/.arduino15/packages/arduino/tools/arm-none-eabi-gcc/4.8.3-2014q1/bin/")
# SET(PLATFORM_TOOLCHAIN_PATH "/opt/Atollic_TrueSTUDIO_for_STM32_x86_64_9.0.0/ARMTools/bin/")
set(PLATFORM_TOOLCHAIN_PATH "/opt/arm_gcc_2017/gcc-arm-none-eabi-7-2017-q4-major/bin/")
# "/home/fx/.arduino15/packages/STM32/tools/arm-none-eabi-gcc/6-2017-q2-update/bin") and here we go
set(CMAKE_TOOLCHAIN_FILE cmake/ArduinoToolchain.cmake)
message(STATUS "Starting hello-stm32")

cmake_minimum_required(VERSION 2.8)
project("Hello_stm32" C CXX ASM)

print_board_list()
add_definitions("-g3 -O0")
add_definitions("-D__CORTEX_M=3") # For FreeRTOS
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/..)
set(ARDUINO_DEFAULT_BOARD genericSTM32F103C) # Default Board ID, when not specified
set(ARDUINO_DEFAULT_PORT ttyACM19) # Default Port, when not specified
set(ARDUINO_CPU STM32F103C8)

# this is needed for the nano, it should be set automatically, maybe something changed with sdk 1.8.x ?
set(ARDUINO_CPUMENU .menu.cpu.${ARDUINO_CPU})
print_settings(${ARDUINO_DEFAULT_BOARD})

# Where you store your downloaded libs or manually installed
set(CUSTOM_LIB_DIR /home/fx/Arduino/libraries)

link_directories(${CUSTOM_LIB_DIR})
#
# Be careful with the includes in the ino files #include <foo.h> // this is a comment will fail same with extra tabs or
# space after the #include "blah.h"
#
set(libPrefix ${ARDUINO_DEFAULT_BOARD}_)
# This reduces size by 10% or so ADD_DEFINITIONS("-ffunction-sections -fdata-sections ")
generate_arduino_firmware(
  mcp
  SKETCH
  test.ino
  SRCS
  myCode.cpp
  ${CMAKE_SOURCE_DIR}/../mcp23017.cpp
  PORT
  ${ARDUINO_DEFAULT_PORT}
  BOARD_CPU
  ${ARDUINO_CPU})

# EOF
