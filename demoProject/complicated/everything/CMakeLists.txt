cmake_minimum_required(VERSION 3.20)

include(../common_CMakeLists.cmake)

project(Longan_SSD1306 C CXX ASM)

include_directories(${CMAKE_SOURCE_DIR})
add_subdirectory(${ESPRIT_ROOT})

include(${ESPRIT_ROOT}/setup.cmake)

set(SSD1306 ${ESPRIT_ROOT}/libraries/simplerSSD1306)
include_directories(${SSD1306})
use_library(WS2812B)

set(LIBSSD ${SSD1306}/ssd1306_base.cpp ${SSD1306}/ssd1306_ex.cpp ${SSD1306}/ssd1306_i2c_lnGd32vf103.cpp)

set(ST7735_DRIVER ${ESPRIT_ROOT}/libraries/simplerST7735/platformExample/longanNano/gd32ST7735.cpp)
include_directories(ST7735_DRIVER ${ESPRIT_ROOT}/libraries/simplerST7735/platformExample/longanNano/)

use_library(simplerST7735)

generate_gd32_firmware(everything hello.cpp ${LIBSSD})
