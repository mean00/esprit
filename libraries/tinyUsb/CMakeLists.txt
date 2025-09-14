
# CFG_TUSB_OS == OPT_OS_FREERTOS

SET(TUSB ${CMAKE_CURRENT_SOURCE_DIR}/src/src)
SET(LNSRC ${CMAKE_CURRENT_SOURCE_DIR}/lnSrc)

include( ${LN_MCU_FOLDER}/tinyUsb/usb_setup.cmake)

IF(NOT DEFINED LN_USB_NB_CDC)
  MESSAGE(FATAL_ERROR "SET LN_USB_NB_CDC to the number of CDC interface you use")
ENDIF(NOT DEFINED LN_USB_NB_CDC)

IF(NOT DEFINED LN_USB_NB_HID)
  MESSAGE(FATAL_ERROR "SET LN_USB_NB_CDC to the number of HID interface you use")
ENDIF(NOT DEFINED LN_USB_NB_HID)

IF( DEFINED LN_USB_DFU_RUNTIME)
  SET(LN_USB_DFU_RUNTIME ${LN_USB_DFU_RUNTIME} CACHE INTERNAL "")
ELSE()
  SET(LN_USB_DFU_RUNTIME 0 CACHE INTERNAL "")
ENDIF()

IF( DEFINED LN_USB_DFU)
  SET(LN_USB_DFU ${LN_USB_DFU} CACHE INTERNAL "")
ELSE()
  SET(LN_USB_DFU 0 CACHE INTERNAL "")
ENDIF()

IF(DEFINED LN_USB_BUFFER_SIZE)
  SET(LN_USB_BUFFER_SIZE ${LN_USB_BUFFER_SIZE} CACHE INTERNAL "")
ELSE()
  SET(LN_USB_BUFFER_SIZE 64 CACHE INTERNAL "")
ENDIF()



configure_file( ${CMAKE_CURRENT_SOURCE_DIR}/tusb_config.h.in ${CMAKE_BINARY_DIR}/tusb_config.h @ONLY)


IF( LN_USB_NB_CDC GREATER 0 )
  SET(CLASS_FILES ${CLASS_FILES} ${TUSB}/class/cdc/cdc_device.c ${LNSRC}/lnUsbCDC.cpp )
ENDIF()
IF( LN_USB_NB_HID GREATER 0 )
  #SET(CLASS_FILES ${CLASS_FILES} ${TUSB}/class/cdc/cdc_device.c
ENDIF()
IF( LN_USB_DFU_RUNTIME GREATER 0 )
  SET(CLASS_FILES ${CLASS_FILES} ${TUSB}/class/dfu/dfu_rt_device.c ${LNSRC}/lnUsbDFUrt.cpp)
ENDIF()

IF( LN_USB_DFU GREATER 0 )
  SET(CLASS_FILES ${CLASS_FILES} ${TUSB}/class/dfu/dfu_rt_device.c ${LNSRC}/lnUsbDFU.cpp)
ENDIF()

IF( LN_USB_DFU GREATER 0 )
  SET(CLASS_FILES ${CLASS_FILES} ${TUSB}/class/dfu/dfu_device.c )
ENDIF()



IF(FALSE)
  ADD_DEFINITIONS("-DCFG_TUSB_DEBUG=2")
  ADD_DEFINITIONS("-DCFG_TUSB_DEBUG_PRINTF=Logger_C")
ENDIF()


SET(SRCS  ${CLASS_FILES}
          ${TUSB}/tusb.c
          ${TUSB}/device/usbd_control.c
          ${TUSB}/device/usbd.c
          ${TUSB}/common/tusb_fifo.c
          ${DRIVERS}
          )
add_library(tinyUsb OBJECT ${SRCS})
target_include_directories(tinyUsb PRIVATE ${CMAKE_BINARY_DIR})
target_include_directories(tinyUsb PRIVATE ${ESPRIT_ROOT}/private_include)
target_include_directories(tinyUsb PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
target_include_directories(tinyUsb PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)
target_include_directories(tinyUsb PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/private_include)
target_include_directories(tinyUsb PUBLIC ${TUSB})
target_include_directories(tinyUsb PUBLIC ${TUSB}/common)
target_include_directories(tinyUsb PUBLIC ${TUSB}/device)
target_include_directories(tinyUsb PUBLIC ${TUSB}/class/device/cdc)
target_include_directories(tinyUsb PUBLIC ${TUSB}/class/device/dfu)

target_link_libraries(tinyUsb esprit_dev)


target_compile_definitions(tinyUsb PUBLIC  "${LN_TUSB_EXTRA_DEF}")
target_include_directories(tinyUsb PRIVATE "${LN_TUSB_EXTRA_IDIR}")
