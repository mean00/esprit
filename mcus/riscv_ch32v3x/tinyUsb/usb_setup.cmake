if(USE_CH32v3x_HW_IRQ_STACK)
  add_definitions("-DUSE_CH32v3x_HW_IRQ_STACK")
endif()

set(LN_OPT_MODE OPT_MODE_FULL_SPEED)
set(LN_OPT_TUSB_MCU OPT_MCU_CH32V307)

if(FALSE) # Old driver
  set(CH32V3x_FOLDER ${LN_MCU_FOLDER}/tinyUsb/old)
  include_directories(${CH32V3x_FOLDER})

  if(USE_CH32v3x_USB_HS)
    message(STATUS "Using old CH32 HS Driver")
    set(DRIVERS ${LNSRC}/lnUsbStack.cpp ${CH32V3x_FOLDER}/dcd_ch32_usbhs.c ${CH32V3x_FOLDER}/../lnUsbStubs.cpp
                ${CH32V3x_FOLDER}/dcd_usbhs_platform.cpp)
  else()
    message(STATUS "Using old CH32 OTG Driver")
    set(DRIVERS
        ${LNSRC}/lnUsbStack.cpp
        # ${CH32V3x_FOLDER}/dcd_usbfs.c
        ${CH32V3x_FOLDER}/dcd_ch32_usbfs.c ${CH32V3x_FOLDER}/dcd_usbfs_platform.cpp ${CH32V3x_FOLDER}/../lnUsbStubs.cpp)
  endif()

else() # New driver

  if(USE_CH32v3x_USB_HS)
    message(STATUS "Using new CH32 HS Driver")
    set(CH32V3x_FOLDER ${LN_MCU_FOLDER}/tinyUsb/HS_new)
    include_directories(${CH32V3x_FOLDER})
    set(DRIVERS ${LNSRC}/lnUsbStack.cpp ${CH32V3x_FOLDER}/dcd_ch32_usbhs.c ${CH32V3x_FOLDER}/dcd_usbhs_platform.cpp
                ${CH32V3x_FOLDER}/../lnUsbStubs.cpp)
  else()
    message(STATUS "Using new CH32 OTG Driver")
    set(CH32V3x_FOLDER ${LN_MCU_FOLDER}/tinyUsb/FS_new)
    include_directories(${CH32V3x_FOLDER})
    set(DRIVERS ${LNSRC}/lnUsbStack.cpp ${CH32V3x_FOLDER}/dcd_ch32_usbfs.c ${CH32V3x_FOLDER}/dcd_usbfs_platform.cpp
                ${CH32V3x_FOLDER}/../lnUsbStubs.cpp)
  endif()

endif()
