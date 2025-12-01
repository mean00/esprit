#

ADD_LIBRARY(ln_utils OBJECT)
TARGET_SOURCES( ln_utils PRIVATE
                       lnFreeRTOS_pp.cpp
                       lnDebug.cpp
                       systemHelper.cpp
                       lnStopWatch.cpp)
TARGET_LINK_LIBRARIES(ln_utils PRIVATE  esprit_dev)
IF(LN_EXTERNAL_MALLOC)
ELSE()
  TARGET_SOURCES( ln_utils PRIVATE
                       systemAlloc.cpp
                       ln_libc.cpp
                    )
ENDIF()

IF(LN_ENABLE_ETH)
  TARGET_SOURCES( ln_utils PRIVATE  lnSocketRunner.cpp)
ENDIF()

If(LN_DEBUG_UART)
  MESSAGE(STATUS "Using uart ${LN_DEBUG_UART} for debug")
  TARGET_COMPILE_DEFINITIONS(ln_utils PRIVATE LN_DEBUG_UART=${LN_DEBUG_UART})
ENDIF()
