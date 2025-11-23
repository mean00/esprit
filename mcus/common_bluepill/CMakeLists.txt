#
INCLUDE_DIRECTORIES(${CMAKE_CURRENT_SOURCE_DIR}/../../private_include)



SET(OVERLAY_SRCS
                       lnSerialCore.cpp
                       lnGPIO.cpp
                       lnDma.cpp
                       lnSystemInit.cpp lnRCU.cpp
                       lnExti.cpp
                       lnBasicTimer.cpp
                       lnBasicDelayTimer.cpp
                       lnDelayTimer.cpp
                       lnTimer.cpp
                       lnDmaTimer.cpp
                       #lnStopWatch.cpp
                       lnDAC.cpp
                       lnADC.cpp
                       lnADC_base.cpp
                       lnADC_timing.cpp
                       lnFMC.cpp
                       lnStubs.cpp

                        lnSerialTxOnly.cpp
                        lnSerialTxOnlyDma.cpp
                        lnSerialTxOnlyDmaBuffer.cpp

#                       ${LN_MCU_FOLDER}/boards/${LN_BOARD_NAME}/lnPinMapping.cpp
                       CACHE INTERNAL ""
                    )




ADD_LIBRARY(esprit_lib OBJECT  ${OVERLAY_SRCS} )
TARGET_LINK_LIBRARIES(esprit_lib  PUBLIC esprit_dev)
TARGET_INCLUDE_DIRECTORIES(esprit_lib PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)

IF(LN_ENABLE_UART)
  target_sources( esprit_lib PRIVATE lnSerialRxTx.cpp lnSerialRxTxDma.cpp  )
  target_compile_options(esprit_lib PRIVATE "-DLN_ENABLE_UART=1")
ENDIF()

MACRO(ALTER nm text)
  IF(${nm})
    MESSAGE(STATUS "Altering ${text} to ${${nm}}")
    TARGET_COMPILE_DEFINITIONS(esprit_lib PRIVATE ${nm}=${${nm}})
  ENDIF()
ENDMACRO()

ALTER(LN_INITIAL_STACK_SIZE "initial stack size")
ALTER(LN_INITIAL_TASK_PRIORITY "initial task priority")


TARGET_COMPILE_DEFINITIONS(esprit_lib PRIVATE LN_MCU_SPEED=${LN_MCU_SPEED})
TARGET_INCLUDE_DIRECTORIES(esprit_lib PRIVATE arduinoLayer/include include FreeRTOS/include . ${LN_MCU_FOLDER}/boards private_include ${LN_MCU_FOLDER}/include)
IF(LN_USE_INTERNAL_CLOCK)
  TARGET_COMPILE_DEFINITIONS(esprit_lib PRIVATE LN_USE_INTERNAL_CLOCK)
ENDIF(LN_USE_INTERNAL_CLOCK)


#


IF(LN_ENABLE_I2C)
  target_sources(esprit_lib PRIVATE lnI2C.cpp lnI2C_impl.cpp  )
ENDIF(LN_ENABLE_I2C)

IF(LN_ENABLE_SPI)
  target_sources(esprit_lib PRIVATE lnSPI_bp.cpp lnSPI_bp_dma.cpp  )
ENDIF(LN_ENABLE_SPI)


#
#
#
