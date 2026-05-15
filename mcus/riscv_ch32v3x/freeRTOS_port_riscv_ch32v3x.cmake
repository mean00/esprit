set(FOS ${ESPRIT_ROOT}/FreeRTOS)
# Enable saving fpu regs ?
if(USE_CH32v3x_HW_IRQ_STACK)
  add_definitions("-DUSE_CH32v3x_HW_IRQ_STACK=1")
endif()
if(USE_HW_FPU)
  add_definitions("-DARCH_FPU=1")
else()
  add_definitions("-DARCH_FPU=0")
endif()
# WCH version
set(LN_FREERTOS_PORT
    ${LN_MCU_FOLDER}/freeRTOS_extension/
    CACHE INTERNAL "")
include_directories(${LN_FREERTOS_PORT}/)
include_directories(${LN_MCU_FOLDER}/)
include_directories(${LN_FREERTOS_PORT}/chip_specific_extensions/RV32I_PFIC_no_extensions)
set(LN_FREERTOS_PORT_SOURCES ${LN_FREERTOS_PORT}/port.c ${LN_FREERTOS_PORT}/portASM.S)
