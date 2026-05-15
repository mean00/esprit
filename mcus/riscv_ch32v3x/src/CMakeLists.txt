include(lnCmake)
set(S ${CMAKE_CURRENT_SOURCE_DIR})

add_subdirectory(../../common_bluepill common_bp)

target_include_directories(esprit_lib PRIVATE ${S} ${S}/../include)
target_include_directories(esprit_dev INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/../../common_bluepill/include
                                                ${CMAKE_CURRENT_SOURCE_DIR}/../boards/${LN_BOARD_NAME})

target_sources(
  esprit_lib
  PRIVATE ${S}/lnCpuID_riscv_ch32v3x.cpp
          ${S}/lnFMC_impl.cpp
          ${S}/lnIRQ_riscv_ch32v3x.cpp
          ${S}/ch32v3x_interrupt_table.cpp
          ${S}/lnRCU_ch32v3x_usb.cpp
          ${S}/lnRCU_systemClock_riscv_ch32v3x.cpp
          ${S}/lnRunTime_riscv_ch32v3x.cpp
          ${S}/lnSystemTimer_riscv_ch32v3x.cpp
          ${S}/lnBoot_riscv_ch32.cpp
          ${S}/../boards/${LN_BOARD_NAME}/lnPinMapping.cpp)
if(LN_XTAL_VALUE)
  target_compile_definitions(esprit_lib PRIVATE CLOCK_XTAL_VALUE=${LN_XTAL_VALUE})
endif()

if(LN_ENABLE_ETH)
  target_compile_definitions(esprit_lib PRIVATE LN_ENABLE_ETH=1)
  target_sources(esprit_lib PRIVATE ${S}/lnRCU_ch32v3x_eth.cpp)
endif()

if(LN_ENABLE_USBD)
  if(USE_CH32v3x_USB_HS)
    target_sources(esprit_lib PRIVATE ${S}/lnUSBD_ch32v3x.cpp)
  else()
    target_sources(esprit_lib PRIVATE ${S}/../../arm_gd32fx/src/lnUSBD.cpp)
  endif()
endif()

if(LN_ENABLE_I2C)
  target_compile_definitions(esprit_lib PRIVATE LN_ENABLE_I2C)
endif()

if(LN_ENABLE_SPI)
  target_compile_definitions(esprit_lib PRIVATE LN_ENABLE_SPI)
endif()

if(USE_CH32V208W)
  target_compile_definitions(esprit_lib PUBLIC USE_CH32V208W)
endif()

if(USE_CH32v3x_HW_IRQ_STACK)
  message(STATUS "Use WCH fast IRQ")
  target_compile_definitions(esprit_lib PRIVATE "-DUSE_CH32v3x_HW_IRQ_STACK")
else()
  message(STATUS "Use WCH slow IRQ")
endif()

if(LN_ENABLE_ETH)
  add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/../lwip lwip_sub)
  target_link_libraries(esprit_lib PRIVATE lnEth_wch)
endif()
