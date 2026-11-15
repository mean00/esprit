# The stack above tinyUSB is Rust when LN_RUST_USB (usb.md): rs_tinyusb owns the
# task, the tud_* callbacks and the descriptors, so lnUsbStack.cpp is not compiled.
# Every other build (non-Rust, and RP2xxx until P7) keeps it.
set(GD32_FOLDER ${LN_MCU_FOLDER}/tinyUsb/)
if(LN_RUST_USB)
  set(LN_USB_STACK_SRC "")
  set(DRIVERS "")
else()
  set(LN_USB_STACK_SRC ${LNSRC}/lnUsbStack.cpp)
  set(DRIVERS ${LN_USB_STACK_SRC} ${GD32_FOLDER}/lnTinyUsbImpl.cpp ${GD32_FOLDER}/lnUsbLL.cpp)
endif()
set(LN_OPT_TUSB_MCU OPT_MCU_STM32F1)
set(LN_OPT_MODE OPT_MODE_FULL_SPEED)
set(LN_TUSB_EXTRA_DEF "STM32F103xB=1")

target_compile_definitions(esprit_dev INTERFACE ${LN_TUSB_EXTRA_DEF})
