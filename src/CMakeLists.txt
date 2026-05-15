#

add_library(ln_utils OBJECT)
target_sources(ln_utils PRIVATE lnFreeRTOS_pp.cpp lnFreeRTOSDebug.cpp lnDebug.cpp systemHelper.cpp lnStopWatch.cpp)
target_link_libraries(ln_utils PRIVATE esprit_dev)
if(LN_EXTERNAL_MALLOC)

else()
  target_sources(ln_utils PRIVATE systemAlloc.cpp ln_libc.cpp)
endif()

if(LN_ENABLE_ETH)
  target_sources(ln_utils PRIVATE lnSocketRunner.cpp)
endif()

if(LN_DEBUG_UART)
  message(STATUS "Using uart ${LN_DEBUG_UART} for debug")
  target_compile_definitions(ln_utils PRIVATE LN_DEBUG_UART=${LN_DEBUG_UART})
endif()
