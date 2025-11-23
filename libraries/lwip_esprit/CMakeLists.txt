#
#
#
#
if ( "${LWIP_DIR}" STREQUAL "")
  message(FATAL_ERROR "LWIP_DIR is not set")
endif()
message(STATUS "LWIP is ${LWIP_DIR}")
set(LWIP_INCLUDE_DIRS
    "${LWIP_DIR}/include"
    "${LWIP_DIR}/src/include"
    "${LWIP_DIR}/contrib/apps/netio"
    "${CMAKE_CURRENT_SOURCE_DIR}/include"
    "${CMAKE_CURRENT_SOURCE_DIR}/include/arch"
)
set(LWIP_ESPRIT ${CMAKE_CURRENT_DIR}/include)

include(${CMAKE_CURRENT_SOURCE_DIR}/Filelists.cmake)
target_sources(lwipcore PRIVATE
  ${CMAKE_CURRENT_SOURCE_DIR}/esprit_lwip.cpp
  ${LWIP_DIR}/contrib/apps/netio/netio.c
  ${LWIP_DIR}/contrib/ports/freertos/sys_arch.c
)
target_include_directories(lwipcore PUBLIC ${LWIP_ESPRIT} ${LWIP_INCLUDE_DIRS})
target_link_libraries(lwipcore PRIVATE esprit_dev)
#target_link_libraries(esprit_dev INTERFACE lwipcore)
