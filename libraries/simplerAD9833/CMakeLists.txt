
SET(SRCS simplerAD9833.cpp )
include_directories(${CMAKE_CURRENT_SOURCE_DIR})
add_library(simplerAD9833 STATIC ${SRCS})
target_include_directories(simplerAD9833 PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(simplerAD9833 PRIVATE esprit_dev)
