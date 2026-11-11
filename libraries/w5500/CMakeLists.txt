#
# W5500 placeholder object library (implementation is in Rust rs_w5500)
#
add_library(w5500 OBJECT ${CMAKE_CURRENT_SOURCE_DIR}/w5500_empty.cpp)

target_include_directories(w5500 PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})

target_link_libraries(w5500 PRIVATE esprit_dev)
