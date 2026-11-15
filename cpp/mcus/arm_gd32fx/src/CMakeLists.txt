include(lnCmake)
set(S ${CMAKE_CURRENT_SOURCE_DIR})
add_subdirectory(../../common_bluepill common_bp)
get_target_property(_esprit_lib_type esprit_lib TYPE)
if(NOT _esprit_lib_type STREQUAL "INTERFACE_LIBRARY")
  target_include_directories(esprit_lib PRIVATE ${S})
endif()
target_include_directories(esprit_dev INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/../../common_bluepill/include
                                                ${CMAKE_CURRENT_SOURCE_DIR}/../boards/${LN_BOARD_NAME})
if(NOT LN_ENABLE_RUST)
  target_sources(
    esprit_lib PRIVATE ${S}/lnCpuID_arm_gd32fx.cpp
                       ${S}/lnIRQ_arm_gd32fx.cpp
                       ${S}/lnRCU_armgd32.cpp
                       ${S}/lnRCU_systemClock_arm_gd32fx.cpp
                       ${S}/lnRunTime_arm_gd32fx.cpp
                       ${S}/lnSystemTimer_arm_gd32fx.cpp
                       ${S}/start.cpp
                       ${S}/../boards/${LN_BOARD_NAME}/lnPinMapping.cpp)
endif()
if(LN_ENABLE_USBD AND NOT (LN_ENABLE_RUST AND LN_RUST_USB))
  target_sources(esprit_lib PRIVATE ${S}/lnUSBD.cpp)
endif()

# The flash-controller entries (FMC_hasFastUnlock, lnFMC::erase/write, ...) have a Rust
# twin in a Rust build - rs_fmc_bluepill, see drop_C.md P7 - so this file is not
# compiled there: it would shadow the crate's exports at link time (the archive is
# scanned before librsbmp_wrapper.a). Every non-Rust build keeps it.
if(NOT LN_ENABLE_RUST)
  target_sources(esprit_lib PRIVATE ${S}/lnFMC_impl.cpp)
endif()

if(NOT _esprit_lib_type STREQUAL "INTERFACE_LIBRARY")
  if(LN_ENABLE_I2C)
    target_compile_definitions(esprit_lib PRIVATE LN_ENABLE_I2C)
  endif()

  if(LN_ENABLE_SPI)
    target_compile_definitions(esprit_lib PRIVATE LN_ENABLE_SPI)
  endif()
endif()

# LN_APPEND_FLAGS( ${S}/lnIRQ_arm_gd32fx.cpp -DLN_ENABLE_I2C) ENDIF(LN_ENABLE_I2C)

# IF(LN_ENABLE_SPI) LN_APPEND_FLAGS(  ${S}/lnIRQ_arm_gd32fx.cpp -DLN_ENABLE_SPI) ENDIF(LN_ENABLE_SPI)
