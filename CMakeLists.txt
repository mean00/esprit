
cmake_minimum_required(VERSION 3.20)

find_program(CCACHE_FOUND ccache)
if(CCACHE_FOUND)
  set(CMAKE_C_COMPILER_LAUNCHER ${CCACHE_FOUND})
  set(CMAKE_CXX_COMPILER_LAUNCHER ${CCACHE_FOUND})
endif()

OPTION(LN_EXTERNAL_FREERTOS "Deactivate internal freertos (esp)")
OPTION(LN_EXTERNAL_MALLOC "Deactivate internal malloc (esp)")
IF(True) #NOT ESPRIT_ROOT)
  IF(NOT ESPRIT_ROOT)
    SET(ESPRIT_ROOT ${CMAKE_CURRENT_SOURCE_DIR} CACHE INTERNAL "")
    LIST(APPEND CMAKE_MODULE_PATH ${ESPRIT_ROOT}/cmake)
  ENDIF()

  MESSAGE(STATUS "ESPRIT_ROOT is ${ESPRIT_ROOT}")

  LIST(APPEND CMAKE_MODULE_PATH  ${ESPRIT_ROOT} ${ESPRIT_ROOT}/cmake)
  include(colorFormatting)

  # Already merged APPLY_PATCH_IF_NEEDED(patch_tinyusb tinyusb_no_static.patch        libraries/tinyUsb      "dont use freertos static constructs ")

  OPTION(LN_ENABLE_USBD "Enable USBD support" OFF)
  OPTION(LN_ENABLE_I2C  "Enable I2C support" ON)
  OPTION(LN_ENABLE_SPI  "Enable SPI support" ON)
  OPTION(LN_ENABLE_UART "Enable Full uart support" OFF)
  OPTION(LN_ENABLE_TIMING_ADC "Enable ADC driver by a timer " ON)

  OPTION(LN_ENABLE_RUST "Enable rust support" OFF)

  # WCH32Vxx specific
  OPTION(USE_CH32v3x_HW_IRQ_STACK "Enable WCH HW interrupt stack push pop (CH32V3xx only)" TRUE)
  OPTION(USE_CH32v3x_FAST_IRQ  "Use the WCH fast IRQ mode (CH32V30x only)" TRUE)
  OPTION(USE_CH32v3x_USB_HS  "Use the USB HS interface (CH32V307 only)" FALSE)
  OPTION(USE_CH32v3x_USB_OTG "Use the USB OTG interface (CH32V30x only)" FALSE)

  # RP2xxx specific
  OPTION(USE_RP2350_PURE_RAM "Load and run from RAM (RP2350 only)" FALSE)
  OPTION(USE_RP2040_PURE_RAM "Load and run from RAM (RP2040 only)" FALSE)
  OPTION(USE_CH32v3x_PURE_RAM "Load and run from RAM (CH32vxx only)" FALSE)

  SET(LN_MCU_FOLDER ${ESPRIT_ROOT}/mcus/${LN_EXT}/ CACHE INTERNAL "")
  #
  ADD_LIBRARY(esprit_dev INTERFACE)

  if(LN_ENABLE_ETH)
    add_subdirectory( libraries/lwip_esprit)
  endif()


  target_include_directories(esprit_dev INTERFACE ${LN_MCU_FOLDER}/boards/${LN_BOARD_NAME}/) # needed to get the freertos_tuning.h
  target_include_directories(esprit_dev INTERFACE ${LN_MCU_FOLDER}/include/) # needed to get the freertos_tuning.h
  target_include_directories(esprit_dev INTERFACE ${ESPRIT_ROOT}/)
  target_include_directories(esprit_dev INTERFACE ${ESPRIT_ROOT}/include)
  target_include_directories(esprit_dev INTERFACE ${CMAKE_BINARY_DIR}) # To catch generated files
  target_include_directories(esprit_dev INTERFACE ${LN_MCU_FOLDER})
  target_include_directories(esprit_dev INTERFACE ${LN_MCU_FOLDER}/include)
  target_include_directories(esprit_dev INTERFACE esprit_dev)
  target_include_directories(esprit_dev INTERFACE embeddedPrintf    )



  IF(LN_BOOTLOADER_SIZE)
    SET(LN_BOOTLOADER_SIZE ${LN_BOOTLOADER_SIZE} CACHE INTERNAL "")
  ELSE(LN_BOOTLOADER_SIZE)
    MESSAGE(STATUS "No bootloader reserved space.")
    SET(LN_BOOTLOADER_SIZE 0 CACHE INTERNAL "")
  ENDIF(LN_BOOTLOADER_SIZE)

  IF(LN_EXTERNAL_FREERTOS)
    target_compile_definitions(esprit_dev INTERFACE LN_CUSTOM_FREERTOS)
  ELSE()
    include(lnFreeRTOS)
    target_include_directories(esprit_dev  INTERFACE FreeRTOS )
  ENDIF()
  include(lnEmbeddedPrint)

  add_subdirectory(${LN_MCU_FOLDER}/src)
  add_subdirectory(${ESPRIT_ROOT}/src)

  IF(LN_ENABLE_RUST)
    add_subdirectory(${ESPRIT_ROOT}/rust) # Generate C++->C for easy Rust binding
  ENDIF(LN_ENABLE_RUST)


  # a firmmware should link against that one
  target_include_directories(esprit_dev INTERFACE ${ESPRIT_ROOT})
  target_include_directories(esprit_dev INTERFACE ${ESPRIT_ROOT}/include)
  target_include_directories(esprit_dev INTERFACE ${LN_MCU_FOLDER})
  target_include_directories(esprit_dev INTERFACE ${ESPRIT_ROOT}/embedded_printf)



  #____________________________________________
  # Merge things into a single lib and single interface
  #____________________________________________

  target_include_directories(esprit_dev INTERFACE embeddedPrintf    )



ENDIF()



