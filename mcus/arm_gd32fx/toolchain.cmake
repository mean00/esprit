#=============================================================================#
MESSAGE(STATUS "Setting up GD32/arm cmake environment")
IF(NOT DEFINED LN_EXT)
  SET(LN_EXT arm_gd32fx CACHE INTERNAL "")
  SET(LN_TOOLCHAIN_EXT  arm_gd32fx CACHE INTERNAL "")
  include(${ESPRIT_ROOT}/../platformConfig.cmake)

  IF(NOT PLATFORM_TOOLCHAIN_PATH)
    MESSAGE(FATAL_ERROR "PLATFORM_TOOLCHAIN_PATH is not defined in platformConfig.cmake !!")
  ENDIF(NOT PLATFORM_TOOLCHAIN_PATH)
  #


  LIST(APPEND CMAKE_SYSTEM_PREFIX_PATH "${PLATFORM_TOOLCHAIN_PATH}")

  FUNCTION(FATAL_BANNER msg)
    MESSAGE(STATUS "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@")
    MESSAGE(STATUS "${msg}")
    MESSAGE(STATUS "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@")
    MESSAGE(FATAL_ERROR "${msg}")
  ENDFUNCTION(FATAL_BANNER msg)

  #
  # Sanity check
  #
  IF(NOT EXISTS "${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}gcc${PLATFORM_TOOLCHAIN_SUFFIX}")
    FATAL_BANNER( "!! PLATFORM_TOOLCHAIN_PATH does not point to a valid toolchain ( ${PLATFORM_PREFIX}gcc....)!! (${PLATFORM_TOOLCHAIN_PATH})")
  ENDIF()
  #
  # Setup toolchain for cross compilation
  #
  SET(CMAKE_SYSTEM_NAME Generic CACHE INTERNAL "")
  SET(CMAKE_C_COMPILER_ID   "GNU" CACHE INTERNAL "")
  SET(CMAKE_CXX_COMPILER_ID "GNU" CACHE INTERNAL "")
  set(CMAKE_C_COMPILER_WORKS      TRUE)
  set(CMAKE_CXX_COMPILER_WORKS    TRUE)
  #
  SET(LN_BOARD_NAME       bluepill CACHE INTERNAL "")
  #SET(LN_LTO "-flto")
  # Speed

  IF(NOT DEFINED LN_MCU_SPEED)
    SET(LN_MCU_SPEED 72000000)
  ENDIF()
  # Set default value
  IF(NOT LN_MCU_RAM_SIZE)
    MESSAGE(STATUS "Ram size not set, using default")
    SET(LN_MCU_RAM_SIZE 20 )
  ENDIF(NOT LN_MCU_RAM_SIZE)
  IF(NOT LN_MCU_FLASH_SIZE)
    MESSAGE(STATUS "Flash size not set, using default")
    SET(LN_MCU_FLASH_SIZE 64 )
  ENDIF(NOT LN_MCU_FLASH_SIZE)
  IF(NOT LN_MCU_EEPROM_SIZE)
    MESSAGE(STATUS "NVME size not set, using default")
    SET(LN_MCU_EEPROM_SIZE 4 )
  ENDIF(NOT LN_MCU_EEPROM_SIZE)
  IF(NOT LN_MCU_STATIC_RAM)
    MESSAGE(STATUS "Static ram size not set, using default")
    SET(LN_MCU_STATIC_RAM 6 )
  ENDIF(NOT LN_MCU_STATIC_RAM)
  #
  SET(LN_MCU_RAM_SIZE ${LN_MCU_RAM_SIZE} CACHE INTERNAL "" FORCE)
  SET(LN_MCU_FLASH_SIZE ${LN_MCU_FLASH_SIZE} CACHE INTERNAL "" FORCE)
  SET(LN_MCU_EEPROM_SIZE ${LN_MCU_EEPROM_SIZE} CACHE INTERNAL "" FORCE)
  SET(LN_MCU_SPEED ${LN_MCU_SPEED} CACHE INTERNAL "" FORCE)
  SET(LN_MCU_STATIC_RAM ${LN_MCU_STATIC_RAM} CACHE INTERNAL "" FORCE)

  IF(USE_SCAN_BUILD)
  ELSE(USE_SCAN_BUILD)
    set(CMAKE_C_COMPILER   ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}gcc${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
    set(CMAKE_ASM_COMPILER ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}gcc${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
    set(CMAKE_CXX_COMPILER ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}g++${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
  ENDIF(USE_SCAN_BUILD)
  set(CMAKE_SIZE         ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}size${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
  set(CMAKE_OBJCOPY      ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}objcopy${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)

  # dont try to create a shared lib, it will not work
  SET(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

  MESSAGE(STATUS "GD32 C   compiler ${CMAKE_C_COMPILER}")
  MESSAGE(STATUS "GD32 C++ compiler ${CMAKE_CXX_COMPILER}")

  # spec files....
  IF(LN_SPEC)
    SET(LN_SPEC "${LN_SPEC}" CACHE INTERNAL "" FORCE)
  ELSE(LN_SPEC)
    SET(LN_SPEC "nano" CACHE INTERNAL "" FORCE)
  ENDIF(LN_SPEC)

  #
  SET(GD32_SPECS  "--specs=${LN_SPEC}.specs")

  # M3 or M4 ?

  IF( "${LN_MCU}" STREQUAL "M3")
    SET(GD32_MCU "  -mcpu=cortex-m3 -mthumb  -march=armv7-m ")
  ELSE()
    IF( "${LN_MCU}" STREQUAL "M4")
      SET(GD32_MCU "-mcpu=cortex-m4  -mfloat-abi=hard -mfpu=fpv4-sp-d16  -mthumb -DLN_USE_FPU=1")
    ELSE()
      MESSAGE(FATAL_ERROR "Unsupported MCU : only M3 is supported (works for M0+) : ${LN_MCU}")
    ENDIF()
  ENDIF()

  SET(G32_DEBUG_FLAGS "-g3 ${LN_LTO}  -O1 " CACHE INTERNAL "")

  SET(GD32_LD_EXTRA "  -Wl,--unresolved-symbols=report-all -Wl,--warn-common -Wl,--warn-section-align " CACHE INTERNAL "")
  #
  SET(GD32_C_FLAGS  "${GD32_SPECS}  ${PLATFORM_C_FLAGS} ${G32_DEBUG_FLAGS} -DLN_ARCH=LN_ARCH_ARM  -Werror=return-type -fmessage-length=0 -fsigned-char -ffunction-sections -fdata-sections -fno-common ${LN_BOARD_NAME_FLAG}  ${GD32_MCU}" CACHE INTERNAL "")
  SET(CMAKE_C_FLAGS "${GD32_C_FLAGS}" CACHE INTERNAL "")
  SET(CMAKE_ASM_FLAGS "${GD32_C_FLAGS}" CACHE INTERNAL "")
  SET(CMAKE_CXX_FLAGS "${GD32_C_FLAGS}  -fno-rtti -fno-exceptions -fno-threadsafe-statics" CACHE INTERNAL "")
  #
  SET(GD32_LD_FLAGS "-nostdlib ${GD32_SPECS} ${GD32_MCU} ${GD32_LD_EXTRA}" CACHE INTERNAL "")
  SET(GD32_LD_LIBS "-lm -lgcc" CACHE INTERNAL "")
  #
  set(CMAKE_CXX_LINK_EXECUTABLE    "<CMAKE_CXX_COMPILER>  <CMAKE_CXX_LINK_FLAGS>  <LINK_FLAGS> ${LN_LTO} -lgcc -Xlinker -print-memory-usage -Xlinker --gc-sections  -Wl,--cref  -Wl,--start-group  <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group  -Wl,--cref -Wl,-Map,<TARGET>.map   -o <TARGET> ${GD32_LD_FLAGS} ${GD32_LD_LIBS}" CACHE INTERNAL "")
  SET(CMAKE_EXECUTABLE_SUFFIX_C .elf CACHE INTERNAL "")
  SET(CMAKE_EXECUTABLE_SUFFIX_CXX .elf CACHE INTERNAL "")


  MESSAGE(STATUS "MCU Architecture ${LN_ARCH}")
  MESSAGE(STATUS "MCU Type         ${LN_MCU}")
  MESSAGE(STATUS "MCU Ext          ${LN_EXT}")
  MESSAGE(STATUS "MCU Speed        ${LN_MCU_SPEED}")
  MESSAGE(STATUS "MCU Flash Size   ${LN_MCU_FLASH_SIZE}")
  MESSAGE(STATUS "NVME Size        ${LN_MCU_EEPROM_SIZE}")
  MESSAGE(STATUS "Bootloader Size  ${LN_BOOTLOADER_SIZE}")
  MESSAGE(STATUS "MCU Ram Size     ${LN_MCU_RAM_SIZE}")
  MESSAGE(STATUS "MCU Static RAM   ${LN_MCU_STATIC_RAM}")

ENDIF(NOT DEFINED LN_EXT)
