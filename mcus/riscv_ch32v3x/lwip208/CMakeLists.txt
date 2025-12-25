#
#
#
#
if ( "${LWIP_DIR}" STREQUAL "")
  message(FATAL_ERROR "LWIP_DIR is not set")
endif()

add_library(lnEthCH32V208 STATIC)
target_sources( lnEthCH32V208 PRIVATE
src/esprit_ch32v208.cpp  src/eth.cpp  src/eth_driver.cpp
)
target_include_directories(lnEthCH32V208 PUBLIC include)
target_include_directories(lnEthCH32V208 PRIVATE private_include
    #"${CMAKE_CURRENT_SOURCE_DIR}/include"
    "${CMAKE_CURRENT_SOURCE_DIR}/private_include"
    "${CMAKE_CURRENT_SOURCE_DIR}/"
)
target_link_libraries(lnEthCH32V208 PRIVATE esprit_dev)
target_include_directories( lnEthCH32V208 PUBLIC ${LWIP_INCLUDE_DIRS} ${ESPRIT_ROOT}/libraries/lwip_esprit/include)
target_link_libraries(lnEthCH32V208 PRIVATE lwipcore)
target_link_libraries(esprit_lib PRIVATE lnEthCH32V208)


