set(RP_CHIP rp2350)
set(RP_CHIP2 RP2350)
add_definitions("-DPICO_RP2350=1")
include(${ESPRIT_ROOT}/mcus/rp_common/tinyUsb/rp_usb_setup.cmake)
