set(FOS ${ESPRIT_ROOT}/FreeRTOS)

if("${LN_MCU}" STREQUAL "M3")
  set(LN_FREERTOS_PORT
      ${FOS}/portable/GCC/ARM_CM3/
      CACHE INTERNAL "")
else()
  if("${LN_MCU}" STREQUAL "M4")
    set(LN_FREERTOS_PORT
        ${FOS}/portable/GCC/ARM_CM4F/
        CACHE INTERNAL "")
  else()
    message(FATAL_ERROR "Unsupported Arch for FreeRTOS ${MCU}")
  endif()
endif()

set(LN_FREERTOS_PORT_SOURCES ${LN_FREERTOS_PORT}/port.c)
