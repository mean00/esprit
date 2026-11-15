# =============================================================================#
message(STATUS "Setting up CH32V3x riscv cmake environment")
if(NOT DEFINED LN_EXT)
  set(LN_EXT
      riscv_ch32v3x
      CACHE INTERNAL "")
  include(${ESPRIT_ROOT}/../platformConfig.cmake)
  set(LN_TOOLCHAIN_EXT
      riscv_ch32v3x
      CACHE INTERNAL "")

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
  set(LN_BOARD_NAME_FLAG
      ""
      CACHE INTERNAL "")
  set(LN_BOARD_NAME
      ch32v3x
      CACHE INTERNAL "")

  if(NOT DEFINED LN_MCU_SPEED)
    set(LN_MCU_SPEED 72000000)
  endif()
  # Set default value
  if(NOT LN_MCU_RAM_SIZE)
    message(STATUS "Ram size not set, using default")
    set(LN_MCU_RAM_SIZE 64)
  endif(NOT LN_MCU_RAM_SIZE)
  if(NOT LN_MCU_FLASH_SIZE)
    message(STATUS "Flash size not set, using default")
    set(LN_MCU_FLASH_SIZE 256)
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

  #

  set(CMAKE_C_COMPILER
      ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}gcc${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)
  set(CMAKE_ASM_COMPILER
      ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}gcc${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)
  set(CMAKE_CXX_COMPILER
      ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}g++${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)
  set(CMAKE_OBJCOPY
      ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}objcopy${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)
  # dont try to create a shared lib, it will not work
  set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

  message(STATUS "CH32V3x C   compiler ${CMAKE_C_COMPILER}")
  message(STATUS "CH32V3x C++ compiler ${CMAKE_CXX_COMPILER}")
  if(LN_SPEC)
    set(LN_SPEC
        "${LN_SPEC}"
        CACHE INTERNAL "" FORCE)
  else(LN_SPEC)
    set(LN_SPEC
        "nano"
        CACHE INTERNAL "" FORCE)
  endif(LN_SPEC)
  set(GD32_SPECS
      "--specs=${LN_SPEC}.specs"
      CACHE INTERNAL "" FORCE)
  message(STATUS "CH32V3x C++ specs   ${LN_SPEC} (${GD32_SPECS})")
  #
  # SET(MCPU " -march=rv32imac -mabi=ilp32 -msmall-data-limit=8 -msave-restore " CACHE INTERNAL "" FORCE)
  set(GD32_DEBUG_FLAGS
      "-g3 -O1 "
      CACHE INTERNAL "")

  #
  set(GD32_SPECS_C_FLAGS
      "${GD32_SPECS}  ${MCPU} ${GD32_DEBUG_FLAGS} ${PLATFORM_C_FLAGS} -DLN_MCU=LN_MCU_CH32V3x -DLN_ARCH=LN_ARCH_RISCV -Werror=return-type  -fmessage-length=0 -fsigned-char -ffunction-sections -fdata-sections -fno-common ${LN_BOARD_NAME_FLAG} -I${ESPRIT_ROOT}/riscv_ch32v3x/"
      CACHE INTERNAL "")
  set(CMAKE_C_FLAGS
      "${GD32_SPECS_C_FLAGS}"
      CACHE INTERNAL "")
  set(CMAKE_AS_FLAGS
      "${PLATFORM_C_FLAGS} ${CMAKE_AS_FLAGS}"
      CACHE INTERNAL "")
  set(CMAKE_ASM_FLAGS
      "${PLATFORM_C_FLAGS} ${CMAKE_ASM_FLAGS}"
      CACHE INTERNAL "")
  set(CMAKE_CXX_FLAGS
      "${GD32_SPECS_C_FLAGS}  -fno-rtti -fno-exceptions -fno-threadsafe-statics"
      CACHE INTERNAL "")
  #
  set(GD32_SPECS_LD_FLAGS
      "-nostdlib ${GD32_SPECS}  -Wl,--traditional-format -Wl,--warn-common"
      CACHE INTERNAL "")
  set(GD32_SPECS_LD_LIBS "-lm  -lgcc")
  set(GD32_LD_LIBS
      "-lm -Wl,--gc-sections "
      CACHE INTERNAL "")

  set(GD32_LD_FLAGS
      ${PLATFORM_C_FLAGS}
      CACHE INTERNAL "")
  #
  set(CMAKE_CXX_LINK_EXECUTABLE
      "<CMAKE_CXX_COMPILER>   <CMAKE_CXX_LINK_FLAGS>  <LINK_FLAGS> ${LN_LTO} -lgcc -Xlinker -print-memory-usage   -Wl,--start-group  <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group  -Wl,-Map,<TARGET>.map   -o <TARGET> ${GD32_SPECS_LD_FLAGS} ${GD32_LD_FLAGS} ${GD32_SPECS_LD_LIBS} ${GD32_LD_LIBS} ${GD32_DEBUG_FLAGS} -lc"
      CACHE INTERNAL "")
  set(CMAKE_EXECUTABLE_SUFFIX_C
      .elf
      CACHE INTERNAL "")
  set(CMAKE_EXECUTABLE_SUFFIX_CXX
      .elf
      CACHE INTERNAL "")

  message(STATUS "MCU Architecture ${LN_ARCH}")
  message(STATUS "MCU Type         ${LN_MCU}")
  message(STATUS "MCU Speed        ${LN_MCU_SPEED}")
  message(STATUS "MCU Flash Size   ${LN_MCU_FLASH_SIZE}")
  message(STATUS "MCU Ram Size     ${LN_MCU_RAM_SIZE}")
  message(STATUS "MCU Static RAM   ${LN_MCU_STATIC_RAM}")
  message(STATUS "LTO Flags        ${LN_LTO}")

endif(NOT DEFINED LN_EXT)
