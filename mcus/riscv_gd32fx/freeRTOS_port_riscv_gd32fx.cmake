set(FOS ${ESPRIT_ROOT}/FreeRTOS)
if(0)
  # Nuclei version
  set(LN_FREERTOS_PORT
      ${ESPRIT_ROOT}/riscv_gd32fx/freeRTOS_extension/N200_nuclei
      CACHE INTERNAL "")
  set(LN_FREERTOS_PORT_SOURCES ${LN_FREERTOS_PORT}/port.c ${LN_FREERTOS_PORT}/portasm.S
                               ${LN_FREERTOS_PORT}/port_interrupt.c ${LN_FREERTOS_PORT}/trap.S)
else()
  # QQ version
  set(LN_FREERTOS_PORT
      ${ESPRIT_ROOT}/riscv_gd32fx/freeRTOS_extension/N200
      CACHE INTERNAL "")
  set(LN_FREERTOS_PORT_SOURCES ${LN_FREERTOS_PORT}/port.c ${LN_FREERTOS_PORT}/portASM.S ${LN_FREERTOS_PORT}/entry.S)
endif()
