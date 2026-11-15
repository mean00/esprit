set(FOS ${ESPRIT_ROOT}/FreeRTOS)
if (ESPRIT_MULTICORE)
    set(LN_FREERTOS_PORT
        ${FOS}/portable/ThirdParty/GCC/RP2040/
        CACHE INTERNAL "")
    set(LN_FREERTOS_PORT_SOURCES ${LN_FREERTOS_PORT}/port.c)
    target_include_directories(esprit_dev INTERFACE ${LN_FREERTOS_PORT}/include)
else()
    set(LN_FREERTOS_PORT
        ${FOS}/portable/GCC/ARM_CM0/
        CACHE INTERNAL "")
    set(LN_FREERTOS_PORT_SOURCES ${LN_FREERTOS_PORT}/port.c ${LN_FREERTOS_PORT}/portasm.c)
endif()
