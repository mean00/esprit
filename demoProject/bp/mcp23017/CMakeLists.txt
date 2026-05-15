cmake_minimum_required(VERSION 3.20)

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
include(common_CMakeLists)

project(Longan_mcp23017 C CXX ASM)

include_directories(${CMAKE_SOURCE_DIR})
add_subdirectory(${ESPRIT_ROOT})

include(${ESPRIT_ROOT}/setup.cmake)

# generic MCP part
set(MCP23017 ${ESPRIT_ROOT}/libraries/mcp23017)
add_subdirectory(${MCP23017})
include_directories(${MCP23017} ${MCP23017}/)

# longan nano specific part
include_directories(${MCP23017} ${MCP23017}/platformExample/longanNano)
set(LIBMCP23017 ${MCP23017}/platformExample/longanNano/lnMcp23017.cpp)
#
generate_gd32_firmware(demo_mcp23017 hello.cpp ${LIBMCP23017})
target_link_libraries(demo_mcp23017 PRIVATE mcp23017)
