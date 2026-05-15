
set(CMAKE_POLICY_DEFAULT_CMP0077 NEW)

cmake_minimum_required(VERSION 3.20)
message(STATUS "======================")
message(STATUS "Starting usb test")
message(STATUS "======================")

set(LN_ENABLE_USBD
    True
    CACHE INTERNAL "")
set(LN_ENABLE_I2C
    FALSE
    CACHE INTERNAL "")
set(LN_ENABLE_SPI
    FALSE
    CACHE INTERNAL "")
set(LN_USB_NB_CDC 1) # 1 CDC interfaces
set(LN_USB_NB_HID 0) # No HID

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
include(common_CMakeLists)

project(lnUsb_usbcdc NONE) # the NONE is very important !

enable_language(C CXX ASM) # this is important too!

#

include_directories(${CMAKE_SOURCE_DIR})
include_directories(${CMAKE_SOURCE_DIR}/INCLUDE)

# SET(LN_ENABLE_RUST ON CACHED "")

add_subdirectory(${ESPRIT_ROOT})

include(${ESPRIT_ROOT}/setup.cmake)
use_library(tinyUsb)

include(${ESPRIT_ROOT}/libraries/tinyUsb/tiny.cmake)

generate_gd32_firmware(hello hello.cpp)
