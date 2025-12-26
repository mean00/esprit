#
#
#
#
if ( "${LWIP_DIR}" STREQUAL "")
  message(FATAL_ERROR "LWIP_DIR is not set")
endif()

add_library(lnEth_wch STATIC)
target_sources( lnEth_wch PRIVATE
  src/ch32v3_eth_driver.cpp
  src/ch32v3_ethernetif.cpp
  src/ch32v30x_eth.cpp
  src/esprit_ch32v3xx.cpp
)
target_include_directories(lnEth_wch PUBLIC include)
target_include_directories(lnEth_wch PRIVATE private_include
    #"${CMAKE_CURRENT_SOURCE_DIR}/include"
    "${CMAKE_CURRENT_SOURCE_DIR}/private_include"
    "${CMAKE_CURRENT_SOURCE_DIR}/"
)
target_link_libraries(lnEth_wch PRIVATE esprit_dev)
target_include_directories( lnEth_wch PUBLIC ${LWIP_INCLUDE_DIRS} ${ESPRIT_ROOT}/libraries/lwip_esprit/include)
target_link_libraries(lnEth_wch PRIVATE lwipcore)
target_link_libraries(esprit_lib PRIVATE lnEth_wch)

if(USE_CH32V208W)
  target_compile_definitions(lnEth_wch PRIVATE  USE_CH32V208W)
endif()

