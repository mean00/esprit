# =============================================================================#
message(STATUS "Setting up GD32 riscv cmake environment")
if(NOT DEFINED LN_EXT)
  set(LN_EXT
      riscv_gd32fx
      CACHE INTERNAL "")
  include(${ESPRIT_ROOT}/../platformConfig.cmake)
  set(LN_TOOLCHAIN_EXT
      riscv_gd32fx
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
  set(LN_BOARD_NAME_FLAG -DGD32VF103C_START) # Longan nano ?
  set(LN_BOARD_NAME
      sipeed-longan-nano
      CACHE INTERNAL "")

  if(NOT DEFINED LN_MCU_SPEED)
    set(LN_MCU_SPEED 72000000)
  endif()
  # Set default value
  if(NOT LN_MCU_RAM_SIZE)
    message(STATUS "Ram size not set, using default")
    set(LN_MCU_RAM_SIZE 32)
  endif(NOT LN_MCU_RAM_SIZE)
  if(NOT LN_MCU_FLASH_SIZE)
    message(STATUS "Flash size not set, using default")
    set(LN_MCU_FLASH_SIZE 128)
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

  message(STATUS "GD32 C   compiler ${CMAKE_C_COMPILER}")
  message(STATUS "GD32 C++ compiler ${CMAKE_CXX_COMPILER}")
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
  #
  set(GD32_C_FLAGS
      "${GD32_SPECS}  ${PLATFORM_C_FLAGS} -DLN_ARCH=LN_ARCH_RISCV -Werror=return-type  -fmessage-length=0 -fsigned-char -ffunction-sections -fdata-sections -fno-common ${LN_BOARD_NAME_FLAG}"
  )
  set(CMAKE_C_FLAGS "${GD32_C_FLAGS}")
  set(CMAKE_CXX_FLAGS "${GD32_C_FLAGS}  -fno-rtti -fno-exceptions -fno-threadsafe-statics")
  #
  set(GD32_LD_FLAGS "-nostdlib ${GD32_SPECS}  -Wl,--traditional-format -Wl,--warn-common")
  set(GD32_LD_LIBS "-lm -lc -lgcc")
  # -lgd32_overlay  -lgd32Arduino -lgd32 -lFreeRTOS -lgd32_lowlevel  -lc -lm -lgd32 -lgcc -lc  -lgcc
  #
  set(CMAKE_CXX_LINK_EXECUTABLE
      "<CMAKE_CXX_COMPILER>   <CMAKE_CXX_LINK_FLAGS>  <LINK_FLAGS> -lgcc -Xlinker -print-memory-usage   -Wl,--start-group  <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group  -Wl,-Map,<TARGET>.map   -o <TARGET> ${GD32_LD_FLAGS} ${GD32_LD_LIBS}"
  )
  set(CMAKE_EXECUTABLE_SUFFIX_C .elf)
  set(CMAKE_EXECUTABLE_SUFFIX_CXX .elf)

  include_directories(${ESPRIT_ROOT}/${LN_EXT}/boards/${LN_BOARD_NAME}/)

  # Sees optimizaton >=2 are causing issues (???)

  add_definitions("-g3  -Os ")
  # ADD_DEFINITIONS("-O0 ")

  message(STATUS "MCU Architecture ${LN_ARCH}")
  message(STATUS "MCU Type         ${LN_MCU}")
  message(STATUS "MCU Speed        ${LN_MCU_SPEED}")
  message(STATUS "MCU Flash Size   ${LN_MCU_FLASH_SIZE}")
  message(STATUS "MCU Ram Size     ${LN_MCU_RAM_SIZE}")
  message(STATUS "MCU Static RAM   ${LN_MCU_STATIC_RAM}")

endif()
