set(SRCS RotaryEncoder.cpp)
include_directories(${CMAKE_CURRENT_SOURCE_DIR})

add_library(RotaryEncoder STATIC ${SRCS})
target_link_libraries(RotaryEncoder PRIVATE esprit_dev)
