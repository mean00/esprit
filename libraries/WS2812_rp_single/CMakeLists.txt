
SET(SRCS lnWS2812_rp_single_spi.cpp lnWS2812_rp_single_pio.cpp)
include_directories(${CMAKE_CURRENT_SOURCE_DIR})
add_library(WS2812_rp_single STATIC ${SRCS})
target_include_directories(WS2812_rp_single PRIVATE ${CMAKE_CURRENT_SOURCe_DIR})
target_link_libraries(WS2812_rp_single PRIVATE esprit_dev esprit_single_lib)
