#
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../private_include)

set(OVERLAY_SRCS
    lnSerialCore.cpp
    lnGPIO.cpp
    lnDma.cpp
    lnSystemInit.cpp
    lnRCU.cpp
    lnExti.cpp
    lnBasicTimer.cpp
    lnBasicDelayTimer.cpp
    lnDelayTimer.cpp
    lnTimer.cpp
    lnDmaTimer.cpp
    lnMultiPulse.cpp
    # lnStopWatch.cpp
    lnDAC.cpp
    lnADC.cpp
    lnADC_base.cpp
    lnADC_timing.cpp
    lnFMC.cpp
    lnStubs.cpp
    lnSerialTxOnly.cpp
    lnSerialTxOnlyDma.cpp
    lnSerialTxOnlyDmaBuffer.cpp
    # ${LN_MCU_FOLDER}/boards/${LN_BOARD_NAME}/lnPinMapping.cpp
    CACHE INTERNAL "")

add_library(esprit_lib OBJECT ${OVERLAY_SRCS})
target_link_libraries(esprit_lib PUBLIC esprit_dev)
target_include_directories(esprit_lib PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)

if(LN_ENABLE_UART)
  target_sources(esprit_lib PRIVATE lnSerialRxTx.cpp lnSerialRxTxDma.cpp)
  target_compile_options(esprit_lib PRIVATE "-DLN_ENABLE_UART=1")
endif()

macro(ALTER nm text)
  if(${nm})
    message(STATUS "Altering ${text} to ${${nm}}")
    target_compile_definitions(esprit_lib PRIVATE ${nm}=${${nm}})
  endif()
endmacro()

alter(LN_INITIAL_STACK_SIZE "initial stack size")
alter(LN_INITIAL_TASK_PRIORITY "initial task priority")

target_compile_definitions(esprit_lib PRIVATE LN_MCU_SPEED=${LN_MCU_SPEED})
target_include_directories(
  esprit_lib
  PRIVATE arduinoLayer/include
          include
          FreeRTOS/include
          .
          ${LN_MCU_FOLDER}/boards
          private_include
          ${LN_MCU_FOLDER}/include)
if(LN_USE_INTERNAL_CLOCK)
  target_compile_definitions(esprit_lib PRIVATE LN_USE_INTERNAL_CLOCK)
endif(LN_USE_INTERNAL_CLOCK)

#

if(LN_ENABLE_I2C)
  target_sources(esprit_lib PRIVATE lnI2C.cpp lnI2C_impl.cpp)
endif(LN_ENABLE_I2C)

if(LN_ENABLE_SPI)
  target_sources(esprit_lib PRIVATE lnSPI_bp.cpp lnSPI_bp_dma.cpp)
endif(LN_ENABLE_SPI)

#
