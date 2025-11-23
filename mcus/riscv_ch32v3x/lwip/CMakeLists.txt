#
#
#
#
if ( "${LWIP_DIR}" STREQUAL "")
  message(FATAL_ERROR "LWIP_DIR is not set")
endif()

add_library(lnEthCh32V3xx STATIC)
target_sources( lnEthCh32V3xx PRIVATE
  src/ch32v3_eth_driver.cpp
  src/ch32v3_ethernetif.cpp
  src/ch32v30x_eth.cpp
  src/esprit_ch32v3xx.cpp
)
target_include_directories(lnEthCh32V3xx PUBLIC include)
target_include_directories(lnEthCh32V3xx PRIVATE private_include
    #"${CMAKE_CURRENT_SOURCE_DIR}/include"
    "${CMAKE_CURRENT_SOURCE_DIR}/private_include"
    "${CMAKE_CURRENT_SOURCE_DIR}/"
)
target_link_libraries(lnEthCh32V3xx PRIVATE esprit_dev)
target_include_directories( lnEthCh32V3xx PUBLIC ${LWIP_INCLUDE_DIRS})
target_link_libraries(lnEthCh32V3xx PRIVATE lwipcore)
target_link_libraries(esprit_lib PRIVATE lnEthCh32V3xx)


