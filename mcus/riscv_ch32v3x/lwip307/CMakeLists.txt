#
#
#
#
if ( "${LWIP_DIR}" STREQUAL "")
  message(FATAL_ERROR "LWIP_DIR is not set")
endif()

add_library(lnEthCH32V307 STATIC)
target_sources( lnEthCH32V307 PRIVATE
  src/ch32v3_eth_driver.cpp
  src/ch32v3_ethernetif.cpp
  src/ch32v30x_eth.cpp
  src/esprit_CH32V307.cpp
)
target_include_directories(lnEthCH32V307 PUBLIC include)
target_include_directories(lnEthCH32V307 PRIVATE private_include
    #"${CMAKE_CURRENT_SOURCE_DIR}/include"
    "${CMAKE_CURRENT_SOURCE_DIR}/private_include"
    "${CMAKE_CURRENT_SOURCE_DIR}/"
)
target_link_libraries(lnEthCH32V307 PRIVATE esprit_dev)
target_include_directories( lnEthCH32V307 PUBLIC ${LWIP_INCLUDE_DIRS} ${ESPRIT_ROOT}/libraries/lwip_esprit/include)
target_link_libraries(lnEthCH32V307 PRIVATE lwipcore)
target_link_libraries(esprit_lib PRIVATE lnEthCH32V307)


