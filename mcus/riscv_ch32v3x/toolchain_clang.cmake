# =============================================================================#
message(STATUS "Setting up CH32V3x riscv cmake environment")
if(NOT DEFINED LN_EXT)
  set(LN_EXT
      riscv_ch32v3x
      CACHE INTERNAL "")
  include(${ESPRIT_ROOT}/../platformConfig.cmake)
  set(LN_TOOLCHAIN_EXT
      riscv_ch32v3x_clang
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
    # FATAL_BANNER( "!! PLATFORM_TOOLCHAIN_PATH does not point to a valid toolchain ( ${PLATFORM_PREFIX}gcc....)!!
    # (${PLATFORM_TOOLCHAIN_PATH})")
  endif()
  #
  # Setup toolchain for cross compilation
  #
  set(CMAKE_SYSTEM_NAME
      Generic
      CACHE INTERNAL "")
  set(CMAKE_C_COMPILER_ID
      "Clang"
      CACHE INTERNAL "")
  set(CMAKE_CXX_COMPILER_ID
      "Clang"
      CACHE INTERNAL "")
  set(CMAKE_C_COMPILER_WORKS
      TRUE
      CACHE INTERNAL "")
  set(CMAKE_CXX_COMPILER_WORKS
      TRUE
      CACHE INTERNAL "")

  #
  set(LN_BOARD_NAME_FLAG
      ""
      CACHE INTERNAL "")
  set(LN_BOARD_NAME
      ch32v3x
      CACHE INTERNAL "")

  if(NOT DEFINED LN_MCU_SPEED)
    set(LN_MCU_SPEED
        72000000
        CACHE INTERNAL "")
  endif()
  # Set default value
  if(NOT LN_MCU_RAM_SIZE)
    message(STATUS "Ram size not set, using default")
    set(LN_MCU_RAM_SIZE
        64
        CACHE INTERNAL "")
  endif(NOT LN_MCU_RAM_SIZE)
  if(NOT LN_MCU_FLASH_SIZE)
    message(STATUS "Flash size not set, using default")
    set(LN_MCU_FLASH_SIZE
        256
        CACHE INTERNAL "")
  endif(NOT LN_MCU_FLASH_SIZE)
  if(NOT LN_MCU_EEPROM_SIZE)
    message(STATUS "NVME size not set, using default")
    set(LN_MCU_EEPROM_SIZE
        4
        CACHE INTERNAL "")
  endif(NOT LN_MCU_EEPROM_SIZE)
  if(NOT LN_MCU_STATIC_RAM)
    message(STATUS "Static ram size not set, using default")
    set(LN_MCU_STATIC_RAM
        6
        CACHE INTERNAL "")
  endif(NOT LN_MCU_STATIC_RAM)
  if(NOT LN_MCU_XTAL_CLOCK)
    message(STATUS "Xtal clock not set,  using default (8 Mhz)")
    set(LN_MCU_XTAL_CLOCK
        8
        CACHE INTERNAL "")
  endif()

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
  if(USE_SCAN_BUILD)

  else(USE_SCAN_BUILD)
    set(CMAKE_C_COMPILER
        ${PLATFORM_CLANG_PATH}/clang${PLATFORM_CLANG_VERSION}${TOOLCHAIN_SUFFIX}
        CACHE PATH "" FORCE)
    set(CMAKE_ASM_COMPILER
        ${PLATFORM_CLANG_PATH}/clang${PLATFORM_CLANG_VERSION}${TOOLCHAIN_SUFFIX}
        CACHE PATH "" FORCE)
    set(CMAKE_CXX_COMPILER
        ${PLATFORM_CLANG_PATH}/clang++${PLATFORM_CLANG_VERSION}${TOOLCHAIN_SUFFIX}
        CACHE PATH "" FORCE)
  endif(USE_SCAN_BUILD)
  set(CMAKE_SIZE
      ${PLATFORM_CLANG_PATH}/llvm-size${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)
  set(CMAKE_AR
      ${PLATFORM_CLANG_PATH}/llvm-ar${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)
  set(CMAKE_RANLIB
      ${PLATFORM_CLANG_PATH}/llvm-ranlib${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)
  set(CMAKE_OBJCOPY
      ${PLATFORM_CLANG_PATH}/llvm-objcopy${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)
  #
  # set(CMAKE_LINKER  ${CMAKE_CXX_COMPILER}                                                             CACHE PATH ""
  # FORCE)

  # dont try to create a shared lib, it will not work
  set(CMAKE_TRY_COMPILE_TARGET_TYPE
      STATIC_LIBRARY
      CACHE INTERNAL "")

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
  set(GD32_DEBUG_FLAGS
      "-g3 -gdwarf-5 ${LN_LTO} -Oz"
      CACHE INTERNAL "")
  #
  # SET(EXTRA_DEBUG "-fno-omit-frame-pointer") SET(EXTRA_DEBUG "-Wdouble-promotion -Werror=double-promotion
  # -fomit-frame-pointer -ffunction-sections -fdata-sections -fno-unwind-tables -fno-asynchronous-unwind-tables")
  set(EXTRA_DEBUG
      "-Wdouble-promotion -Werror=double-promotion -fno-omit-frame-pointer -ffunction-sections -fdata-sections -fno-unwind-tables -fno-asynchronous-unwind-tables"
  )
  #
  set(GD32_MCU_C_FLAGS
      "--sysroot ${PLATFORM_CLANG_SYSROOT} ${EXTRA_DEBUG} ${PLATFORM_CLANG_C_FLAGS} -DLN_MCU=LN_MCU_CH32V3x -DLN_ARCH=LN_ARCH_RISCV ${LN_BOARD_NAME_FLAG} -I${ESPRIT_ROOT}/riscv_ch32v3x/"
      CACHE INTERNAL "")
  set(GD32_C_FLAGS
      "-DLN_MCU_XTAL_CLOCK=${LN_MCU_XTAL_CLOCK} ${GD32_SPECS_SPECS} ${GD32_MCU_C_FLAGS} ${GD32_DEBUG_FLAGS}  -Werror=return-type  -fmessage-length=0 -fsigned-char -ffunction-sections -fdata-sections -fno-common "
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
  set(CLANG_LINKER_OPT
      ""
      CACHE INTERNAL "")
  #
  set(GD32_LD_FLAGS
      "-fuse-ld=lld  -L${PLATFORM_CLANG_SYSROOT}/lib/riscv32-unknown-unknown-elf ${LN_LTO} -nostartfiles -nostdlib ${GD32_SPECS_SPECS} --sysroot ${PLATFORM_CLANG_SYSROOT}  -Wl,--warn-common"
      CACHE INTERNAL "")
  set(GD32_LD_LIBS
      "-lm   ${CLANG_LINKER_OPT} -Wl,--gc-sections -ffunction-sections -fdata-sections  -Wl,--gdb-index "
      CACHE INTERNAL "")

  #
  set(CMAKE_CXX_LINK_EXECUTABLE
      "<CMAKE_CXX_COMPILER>  ${PLATFORM_CLANG_EXTRA_LD_ARG} <CMAKE_CXX_LINK_FLAGS>  ${PLATFORM_CLANG_C_FLAGS} <LINK_FLAGS>   -Wl,--start-group  <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group  -Wl,-Map,<TARGET>.map   -o <TARGET> ${GD32_LD_FLAGS} ${GD32_LD_LIBS} ${GD32_LD_FLAGS}   -lclang_rt.builtins "
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
  message(STATUS "MCU Crystal clock ${LN_MCU_XTAL_CLOCK}")
  message(STATUS "CFlags           ${PLATFORM_CLANG_C_FLAGS}")

endif(NOT DEFINED LN_EXT)
