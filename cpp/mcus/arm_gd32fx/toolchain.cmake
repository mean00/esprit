# =============================================================================#
message(STATUS "Setting up GD32/arm cmake environment")
if(NOT DEFINED LN_EXT)
  set(LN_EXT
      arm_gd32fx
      CACHE INTERNAL "")
  set(LN_TOOLCHAIN_EXT
      arm_gd32fx
      CACHE INTERNAL "")
  include(${ESPRIT_ROOT}/../platformConfig.cmake)

  if(NOT PLATFORM_TOOLCHAIN_PATH)
    message(FATAL_ERROR "PLATFORM_TOOLCHAIN_PATH is not defined in platformConfig.cmake !!")
  endif(NOT PLATFORM_TOOLCHAIN_PATH)
  #

  list(APPEND CMAKE_SYSTEM_PREFIX_PATH "${PLATFORM_TOOLCHAIN_PATH}")

  function(FATAL_BANNER msg)
    message(STATUS "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@")
    message(STATUS "${msg}")
    message(STATUS "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@")
    message(FATAL_ERROR "${msg}")
  endfunction(FATAL_BANNER msg)

  #
  # Sanity check
  #
  if(NOT EXISTS "${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}gcc${PLATFORM_TOOLCHAIN_SUFFIX}")
    fatal_banner(
      "!! PLATFORM_TOOLCHAIN_PATH does not point to a valid toolchain ( ${PLATFORM_PREFIX}gcc....)!! (${PLATFORM_TOOLCHAIN_PATH})"
    )
  endif()
  #
  # Setup toolchain for cross compilation
  #
  set(CMAKE_SYSTEM_NAME
      Generic
      CACHE INTERNAL "")
  set(CMAKE_C_COMPILER_ID
      "GNU"
      CACHE INTERNAL "")
  set(CMAKE_CXX_COMPILER_ID
      "GNU"
      CACHE INTERNAL "")
  set(CMAKE_C_COMPILER_WORKS TRUE)
  set(CMAKE_CXX_COMPILER_WORKS TRUE)
  #
  set(LN_BOARD_NAME
      bluepill
      CACHE INTERNAL "")
  # SET(LN_LTO "-flto") Speed

  if(NOT DEFINED LN_MCU_SPEED)
    set(LN_MCU_SPEED 72000000)
  endif()
  # Set default value
  if(NOT LN_MCU_RAM_SIZE)
    message(STATUS "Ram size not set, using default")
    set(LN_MCU_RAM_SIZE 20)
  endif(NOT LN_MCU_RAM_SIZE)
  if(NOT LN_MCU_FLASH_SIZE)
    message(STATUS "Flash size not set, using default")
    set(LN_MCU_FLASH_SIZE 64)
  endif(NOT LN_MCU_FLASH_SIZE)
  if(NOT LN_MCU_EEPROM_SIZE)
    message(STATUS "NVME size not set, using default")
    set(LN_MCU_EEPROM_SIZE 4)
  endif(NOT LN_MCU_EEPROM_SIZE)
  if(NOT LN_MCU_STATIC_RAM)
    message(STATUS "Static ram size not set, using default")
    set(LN_MCU_STATIC_RAM 6)
  endif(NOT LN_MCU_STATIC_RAM)
  #
  set(LN_MCU_RAM_SIZE
      ${LN_MCU_RAM_SIZE}
      CACHE INTERNAL "" FORCE)
  set(LN_MCU_FLASH_SIZE
      ${LN_MCU_FLASH_SIZE}
      CACHE INTERNAL "" FORCE)
  set(LN_MCU_EEPROM_SIZE
      ${LN_MCU_EEPROM_SIZE}
      CACHE INTERNAL "" FORCE)
  set(LN_MCU_SPEED
      ${LN_MCU_SPEED}
      CACHE INTERNAL "" FORCE)
  set(LN_MCU_STATIC_RAM
      ${LN_MCU_STATIC_RAM}
      CACHE INTERNAL "" FORCE)

  if(USE_SCAN_BUILD)

  else(USE_SCAN_BUILD)
    set(CMAKE_C_COMPILER
        ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}gcc${TOOLCHAIN_SUFFIX}
        CACHE PATH "" FORCE)
    set(CMAKE_ASM_COMPILER
        ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}gcc${TOOLCHAIN_SUFFIX}
        CACHE PATH "" FORCE)
    set(CMAKE_CXX_COMPILER
        ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}g++${TOOLCHAIN_SUFFIX}
        CACHE PATH "" FORCE)
  endif(USE_SCAN_BUILD)
  set(CMAKE_SIZE
      ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}size${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)
  set(CMAKE_OBJCOPY
      ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}objcopy${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)

  # dont try to create a shared lib, it will not work
  set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

  message(STATUS "GD32 C   compiler ${CMAKE_C_COMPILER}")
  message(STATUS "GD32 C++ compiler ${CMAKE_CXX_COMPILER}")

  # spec files....
  if(LN_SPEC)
    set(LN_SPEC
        "${LN_SPEC}"
        CACHE INTERNAL "" FORCE)
  else(LN_SPEC)
    set(LN_SPEC
        "nano"
        CACHE INTERNAL "" FORCE)
  endif(LN_SPEC)

  #
  set(GD32_SPECS "--specs=${LN_SPEC}.specs")

  # M3 or M4 ?

  if("${LN_MCU}" STREQUAL "M3")
    set(GD32_MCU "  -mcpu=cortex-m3 -mthumb  -march=armv7-m ")
  else()
    if("${LN_MCU}" STREQUAL "M4")
      set(GD32_MCU "-mcpu=cortex-m4  -mfloat-abi=hard -mfpu=fpv4-sp-d16  -mthumb -DLN_USE_FPU=1")
    else()
      message(FATAL_ERROR "Unsupported MCU : only M3 is supported (works for M0+) : ${LN_MCU}")
    endif()
  endif()

  set(G32_DEBUG_FLAGS
      "-g3 ${LN_LTO}  -O1 "
      CACHE INTERNAL "")

  set(GD32_LD_EXTRA
      "  -Wl,--unresolved-symbols=report-all -Wl,--warn-common -Wl,--warn-section-align "
      CACHE INTERNAL "")
  #
  set(GD32_C_FLAGS
      "${GD32_SPECS}  ${PLATFORM_C_FLAGS} ${G32_DEBUG_FLAGS} -DLN_ARCH=LN_ARCH_ARM  -Werror=return-type -fmessage-length=0 -fsigned-char -ffunction-sections -fdata-sections -fno-common ${LN_BOARD_NAME_FLAG}  ${GD32_MCU}"
      CACHE INTERNAL "")
  set(CMAKE_C_FLAGS
      "${GD32_C_FLAGS}"
      CACHE INTERNAL "")
  set(CMAKE_ASM_FLAGS
      "${GD32_C_FLAGS}"
      CACHE INTERNAL "")
  set(CMAKE_CXX_FLAGS
      "${GD32_C_FLAGS}  -fno-rtti -fno-exceptions -fno-threadsafe-statics"
      CACHE INTERNAL "")
  #
  set(GD32_LD_FLAGS
      "-nostdlib ${GD32_SPECS} ${GD32_MCU} ${GD32_LD_EXTRA}"
      CACHE INTERNAL "")
  set(GD32_LD_LIBS
      "-lm -lgcc"
      CACHE INTERNAL "")
  #
  set(CMAKE_CXX_LINK_EXECUTABLE
      "<CMAKE_CXX_COMPILER>  <CMAKE_CXX_LINK_FLAGS>  <LINK_FLAGS> ${LN_LTO} -lgcc -Xlinker -print-memory-usage -Xlinker --gc-sections  -Wl,--cref  -Wl,--start-group  <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group  -Wl,--cref -Wl,-Map,<TARGET>.map   -o <TARGET> ${GD32_LD_FLAGS} ${GD32_LD_LIBS}"
      CACHE INTERNAL "")
  set(CMAKE_EXECUTABLE_SUFFIX_C
      .elf
      CACHE INTERNAL "")
  set(CMAKE_EXECUTABLE_SUFFIX_CXX
      .elf
      CACHE INTERNAL "")

  message(STATUS "MCU Architecture ${LN_ARCH}")
  message(STATUS "MCU Type         ${LN_MCU}")
  message(STATUS "MCU Ext          ${LN_EXT}")
  message(STATUS "MCU Speed        ${LN_MCU_SPEED}")
  message(STATUS "MCU Flash Size   ${LN_MCU_FLASH_SIZE}")
  message(STATUS "NVME Size        ${LN_MCU_EEPROM_SIZE}")
  message(STATUS "Bootloader Size  ${LN_BOOTLOADER_SIZE}")
  message(STATUS "MCU Ram Size     ${LN_MCU_RAM_SIZE}")
  message(STATUS "MCU Static RAM   ${LN_MCU_STATIC_RAM}")

endif(NOT DEFINED LN_EXT)
