
SET(SRCS simplerMCP4725.cpp )
include_directories(${CMAKE_CURRENT_SOURCE_DIR})
add_library(simplerMCP4725 STATIC ${SRCS})
target_include_directories(simplerMCP4725 PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(simplerMCP4725 PRIVATE esprit_dev)
