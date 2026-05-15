cmake_minimum_required(VERSION 3.20)

find_program(CCACHE_FOUND ccache)
if(CCACHE_FOUND)
  set(CMAKE_C_COMPILER_LAUNCHER ${CCACHE_FOUND})
  set(CMAKE_CXX_COMPILER_LAUNCHER ${CCACHE_FOUND})
endif()

option(LN_EXTERNAL_FREERTOS "Deactivate internal freertos (esp)")
option(LN_EXTERNAL_MALLOC "Deactivate internal malloc (esp)")
if(True) # NOT ESPRIT_ROOT)
  if(NOT ESPRIT_ROOT)
    set(ESPRIT_ROOT
        ${CMAKE_CURRENT_SOURCE_DIR}
        CACHE INTERNAL "")
    list(APPEND CMAKE_MODULE_PATH ${ESPRIT_ROOT}/cmake)
  endif()

  message(STATUS "ESPRIT_ROOT is ${ESPRIT_ROOT}")

  list(APPEND CMAKE_MODULE_PATH ${ESPRIT_ROOT} ${ESPRIT_ROOT}/cmake)
  include(colorFormatting)

  # Already merged APPLY_PATCH_IF_NEEDED(patch_tinyusb tinyusb_no_static.patch        libraries/tinyUsb      "dont use
  # freertos static constructs ")
  option(LN_ENABLE_FLOAT_PRINTF "Enable float printf" OFF)

  option(LN_ENABLE_USBD "Enable USBD support" OFF)
  option(LN_ENABLE_I2C "Enable I2C support" ON)
  option(LN_ENABLE_SPI "Enable SPI support" ON)
  option(LN_ENABLE_UART "Enable Full uart support" OFF)
  option(LN_ENABLE_TIMING_ADC "Enable ADC driver by a timer " ON)

  option(LN_ENABLE_RUST "Enable rust support" OFF)
  option(LN_ENABLE_ETH "Enable Ethernet support" OFF)

  # WCH32Vxx specific
  option(USE_CH32v3x_HW_IRQ_STACK "Enable WCH HW interrupt stack push pop (CH32V3xx only)" TRUE)
  option(USE_CH32v3x_FAST_IRQ "Use the WCH fast IRQ mode (CH32V30x only)" TRUE)
  option(USE_CH32v3x_USB_HS "Use the USB HS interface (CH32V307 only)" FALSE)
  option(USE_CH32v3x_USB_OTG "Use the USB OTG interface (CH32V30x only)" FALSE)

  # RP2xxx specific
  option(USE_RP2350_PURE_RAM "Load and run from RAM (RP2350 only)" FALSE)
  option(USE_RP2040_PURE_RAM "Load and run from RAM (RP2040 only)" FALSE)
  option(USE_CH32v3x_PURE_RAM "Load and run from RAM (CH32vxx only)" FALSE)

  set(LN_MCU_FOLDER
      ${ESPRIT_ROOT}/mcus/${LN_EXT}/
      CACHE INTERNAL "")
  #
  add_library(esprit_dev INTERFACE)

  if(LN_ENABLE_ETH)
    add_subdirectory(libraries/lwip_esprit)
  endif()

  target_include_directories(esprit_dev INTERFACE ${LN_MCU_FOLDER}/boards/${LN_BOARD_NAME}/) # needed to get the
                                                                                             # freertos_tuning.h
  target_include_directories(esprit_dev INTERFACE ${LN_MCU_FOLDER}/include/) # needed to get the freertos_tuning.h
  target_include_directories(esprit_dev INTERFACE ${ESPRIT_ROOT}/)
  target_include_directories(esprit_dev INTERFACE ${ESPRIT_ROOT}/include)
  target_include_directories(esprit_dev INTERFACE ${CMAKE_BINARY_DIR}) # To catch generated files
  target_include_directories(esprit_dev INTERFACE ${LN_MCU_FOLDER})
  target_include_directories(esprit_dev INTERFACE ${LN_MCU_FOLDER}/include)
  target_include_directories(esprit_dev INTERFACE esprit_dev)
  target_include_directories(esprit_dev INTERFACE embeddedPrintf)

  if(LN_BOOTLOADER_SIZE)
    set(LN_BOOTLOADER_SIZE
        ${LN_BOOTLOADER_SIZE}
        CACHE INTERNAL "")
  else(LN_BOOTLOADER_SIZE)
    message(STATUS "No bootloader reserved space.")
    set(LN_BOOTLOADER_SIZE
        0
        CACHE INTERNAL "")
  endif(LN_BOOTLOADER_SIZE)

  if(LN_EXTERNAL_FREERTOS)
    target_compile_definitions(esprit_dev INTERFACE LN_CUSTOM_FREERTOS)
  else()
    include(lnFreeRTOS)
    target_include_directories(esprit_dev INTERFACE FreeRTOS)
  endif()
  include(lnEmbeddedPrint)

  add_subdirectory(${LN_MCU_FOLDER}/src)
  add_subdirectory(${ESPRIT_ROOT}/src)

  if(LN_ENABLE_RUST)
    add_subdirectory(${ESPRIT_ROOT}/rust) # Generate C++->C for easy Rust binding
  endif(LN_ENABLE_RUST)

  # a firmmware should link against that one
  target_include_directories(esprit_dev INTERFACE ${ESPRIT_ROOT})
  target_include_directories(esprit_dev INTERFACE ${ESPRIT_ROOT}/include)
  target_include_directories(esprit_dev INTERFACE ${LN_MCU_FOLDER})
  target_include_directories(esprit_dev INTERFACE ${ESPRIT_ROOT}/embedded_printf)

  # ____________________________________________ Merge things into a single lib and single interface
  # ____________________________________________

  target_include_directories(esprit_dev INTERFACE embeddedPrintf)

endif()
