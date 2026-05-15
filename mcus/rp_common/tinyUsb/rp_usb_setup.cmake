set(RP2040_FOLDER ${LN_MCU_FOLDER}/tinyUsb/)
set(PICO_USB_FOLDER ${PICO_SDK_PATH}/lib/tinyusb/src/portable/raspberrypi/rp2040/)
set(NATIVE_DRIVER ${ESPRIT_ROOT}/libraries/tinyUsb/src/src/portable/raspberrypi/rp2040)

set(RP_COMMON_FOLDER ${ESPRIT_ROOT}/mcus/rp_common/tinyUsb/)

set(DRIVERS ${LNSRC}/lnUsbStack.cpp ${RP_COMMON_FOLDER}/rp2040_usb.cpp ${RP_COMMON_FOLDER}/lnUSBD_rp2040.cpp
            ${NATIVE_DRIVER}/dcd_rp2040.c ${NATIVE_DRIVER}/rp2040_usb.c)
set(LN_OPT_TUSB_MCU OPT_MCU_RP2040)
set(LN_OPT_MODE OPT_MODE_FULL_SPEED)

#
foreach(
  subdir
  hardware_uart
  hardware_sync
  hardware_gpio
  pico_stdio
  hardware_timer
  hardware_resets
  hardware_irq
  hardware_base)
  list(APPEND idir ${PICO_SDK_PATH}/src/rp2_common/${subdir}/include)
endforeach()

foreach(subdir pico_platform)
  list(APPEND idir ${PICO_SDK_PATH}/src/${RP_CHIP}/${subdir}/include)
endforeach()
#
foreach(subdir pico_platform_compiler pico_platform_sections pico_platform_panic hardware_sync_spin_lock
               boot_bootrom_headers)
  list(APPEND idir ${PICO_SDK_PATH}/src/rp2_common/${subdir}/include)
endforeach()
#
list(APPEND idir ${PICO_SDK_PATH}/src/common/pico_base_headers/include/
     ${PICO_SDK_PATH}/src/${RP_CHIP}/pico_platform/include/) # for sdk2

foreach(
  subdir
  /
  src/common/pico_time/include/
  src/common/pico_base/include/
  src/common/pico_stdlib/include/pico/
  src/${RP_CHIP}/hardware_structs/include/
  src/${RP_CHIP}/hardware_regs/include
  src/boards/include/boards/)
  list(APPEND idir ${PICO_SDK_PATH}/${subdir})
endforeach()

set(LN_TUSB_EXTRA_IDIR ${idir})

# ${LNSRC}/lnUsbStubs.cpp
