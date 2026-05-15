set(RP_CHIP rp2040)
set(RP_CHIP2 RP2040)
add_definitions("-DPICO_RP2040=1")
include(${ESPRIT_ROOT}/mcus/rp_common/tinyUsb/rp_usb_setup.cmake)
