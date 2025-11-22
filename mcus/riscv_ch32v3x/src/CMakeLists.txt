include(lnCmake)
SET(S ${CMAKE_CURRENT_SOURCE_DIR})


add_subdirectory(../../common_bluepill common_bp)

target_include_directories(esprit_lib PRIVATE ${S} ${S}/../include)
target_include_directories(esprit_dev  INTERFACE  ${CMAKE_CURRENT_SOURCE_DIR}/../../common_bluepill/include
        ${CMAKE_CURRENT_SOURCE_DIR}/../boards/${LN_BOARD_NAME})

target_sources( esprit_lib PRIVATE
    ${S}/lnCpuID_riscv_ch32v3x.cpp
    ${S}/lnFMC_impl.cpp
    ${S}/lnIRQ_riscv_ch32v3x.cpp
    ${S}/ch32v3x_interrupt_table.cpp
    ${S}/lnRCU_ch32v3x_usb.cpp
    ${S}/lnRCU_systemClock_riscv_ch32v3x.cpp
    ${S}/lnRunTime_riscv_ch32v3x.cpp
    ${S}/lnSystemTimer_riscv_ch32v3x.cpp
    ${S}/lnBoot_riscv_ch32.cpp
    ${S}/../boards/${LN_BOARD_NAME}/lnPinMapping.cpp
)
IF(LN_XTAL_VALUE)
  target_compile_definitions(esprit_lib PRIVATE CLOCK_XTAL_VALUE=${LN_XTAL_VALUE})
endif()

IF(LN_ENABLE_ETH)
  target_compile_definitions(esprit_lib PRIVATE LN_ENABLE_ETH=1)
  target_sources(esprit_lib PRIVATE ${S}/lnRCU_ch32v3x_eth.cpp)
endif()

IF(LN_ENABLE_USBD)
  target_sources( esprit_lib PRIVATE ${S}/lnUSBD_ch32v3x.cpp)
ENDIF()

IF(LN_ENABLE_I2C)
  target_compile_definitions(esprit_lib PRIVATE LN_ENABLE_I2C )
ENDIF()

IF(LN_ENABLE_SPI)
  target_compile_definitions(esprit_lib PRIVATE LN_ENABLE_SPI )
ENDIF()

IF(USE_CH32V208W)
  target_compile_definitions(esprit_lib PUBLIC USE_CH32V208W )
ENDIF()

IF(USE_CH32v3x_HW_IRQ_STACK)
  MESSAGE(STATUS "Use WCH fast IRQ")
  target_compile_definitions(esprit_lib PRIVATE "-DUSE_CH32v3x_HW_IRQ_STACK")
ELSE()
  MESSAGE(STATUS "Use WCH slow IRQ")
ENDIF()

