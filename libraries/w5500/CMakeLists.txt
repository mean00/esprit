#
#
#
add_library(w5500 STATIC)

target_sources(w5500 PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/lowlevel_w5500.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/socket_w5500.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/socket_high_w5500.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/network_w5500.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/dhcp_w5500.cpp
)
target_include_directories(w5500 PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
target_include_directories(w5500 PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/W5500)

#target_compile_options(w5500 PRIVATE -D_DHCP_DEBUG_=1)

target_compile_options(w5500 PRIVATE -D_WIZCHIP_=5500)

target_link_libraries(w5500 PRIVATE esprit_dev)
#
#
#
