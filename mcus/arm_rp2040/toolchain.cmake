# =============================================================================#
# https://interrupt.memfault.com/blog/arm-cortexm-with-llvm-clang
message(STATUS "Setting up RP2040/arm cmake environment")
if(NOT DEFINED LN_EXT)
  set(LN_EXT
      arm_rp2040
      CACHE INTERNAL "")
  set(LN_TOOLCHAIN_EXT
      arm_rp2040_clang
      CACHE INTERNAL "")

  include(${ESPRIT_ROOT}/../platformConfig.cmake)

  if(NOT PLATFORM_TOOLCHAIN_PATH)
    message(FATAL_ERROR "PLATFORM_TOOLCHAIN_PATH is not defined in platformConfig.cmake !!")
  endif()
  #
  # SET(LN_LTO "-flto")
  #
  list(APPEND CMAKE_SYSTEM_PREFIX_PATH "${PLATFORM_TOOLCHAIN_PATH}")

  function(FATAL_BANNER msg)
    message(STATUS "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@")
    message(STATUS "${msg}")
    message(STATUS "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@")
    message(FATAL_ERROR "${msg}")
  endfunction()

  #
  # Sanity check
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
      rp2040
      CACHE INTERNAL "")

  # Speed
  if(USE_SCAN_BUILD)

  else()
    set(CMAKE_C_COMPILER
        ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}gcc${TOOLCHAIN_SUFFIX}
        CACHE PATH "" FORCE)
    set(CMAKE_ASM_COMPILER
        ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}gcc${TOOLCHAIN_SUFFIX}
        CACHE PATH "" FORCE)
    set(CMAKE_CXX_COMPILER
        ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}g++${TOOLCHAIN_SUFFIX}
        CACHE PATH "" FORCE)
  endif()

  set(CMAKE_SIZE
      ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}size${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)
  set(CMAKE_OBJCOPY
      ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}objcopy${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)
  set(CMAKE_OBJDUMP
      ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}objdump${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)
  set(CMAKE_AR
      ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}ar${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)
  set(CMAKE_RANLIB
      ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}ranlib${TOOLCHAIN_SUFFIX}
      CACHE PATH "" FORCE)

  # We use gcc linker, no we dont
  set(CMAKE_LINKER
      ${CMAKE_CXX_COMPILER}
      CACHE PATH "" FORCE)

  # dont try to create a shared lib, it will not work
  set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

  # OK, now setup the pico sdk
  # ---------------------------------------------------------
  set(PICO_COMPILER
      pico_arm_clang
      CACHE INTERNAL "")
  set(PICO_TOOLCHAIN_PATH
      /arm/tools_llvm/bin
      CACHE INTERNAL "")
  set(LD_SCRIPT
      ${CMAKE_BINARY_DIR}/linker_script.ld
      CACHE INTERNAL "")
  set(PICO_BOARD "pico")
  set(PICO_COMPILER
      pico_arm_clang
      CACHE INTERNAL "")
  set(PICO_TOOLCHAIN_PATH
      ${PLATFORM_CLANG_PATH}
      CACHE INTERNAL "")

  # ---------------------------------------------------------

  if(NOT DEFINED LN_MCU_SPEED)
    set(LN_MCU_SPEED 125000000)
  endif()

  # Set default value
  if(NOT LN_MCU_RAM_SIZE)
    message(STATUS "Ram size not set, using default")
    set(LN_MCU_RAM_SIZE 230)
  endif()
  if(NOT LN_MCU_FLASH_SIZE)
    message(STATUS "Flash size not set, using default")
    set(LN_MCU_FLASH_SIZE 4000)
  endif()
  if(NOT LN_MCU_EEPROM_SIZE)
    message(STATUS "NVME size not set, using default")
    set(LN_MCU_EEPROM_SIZE 4)
  endif()
  if(NOT LN_MCU_STATIC_RAM)
    message(STATUS "Static ram size not set, using default")
    set(LN_MCU_STATIC_RAM 6)
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
  set(LN_MCU_STATIC_RAM
      ${LN_MCU_STATIC_RAM}
      CACHE INTERNAL "" FORCE)

  message(STATUS "GD32 C   compiler ${CMAKE_C_COMPILER}")
  message(STATUS "GD32 C++ compiler ${CMAKE_CXX_COMPILER}")

  # M0+

  set(GD32_LIBC
      "-L${MINI_SYSROOT}/lib "
      CACHE INTERNAL "")
  set(GD32_MCU
      "-mcpu=cortex-m0plus  -DUSE_RP2040  -DPICO_RP2040_USB_FAST_IRQ=0  "
      CACHE INTERNAL "")

  # ____________________________ CLANG LINK USING GCC LIBS ____________________________ SET(LN_CLANG_LIBC
  # /home/fx/Arduino_stm32/arm-gcc-2020q4/bin/../lib/gcc/arm-none-eabi/10.2.1/thumb/v7-m/nofp/libgcc.a CACHE INTERNAL
  # "") SET(LN_CLANG_MULTILIB thumb/v7-m/nofp CACHE INTERNAL "")

  # SET(G32_DEBUG_FLAGS "-g3 ${LN_LTO}  -O0  -gdwarf-4" CACHE INTERNAL "")
  set(G32_DEBUG_FLAGS
      "-g3 ${LN_LTO}  -O1 -gdwarf-4"
      CACHE INTERNAL "")

  set(GD32_LD_EXTRA
      "  -Wl,--unresolved-symbols=report-all -Wl,--warn-common  "
      CACHE INTERNAL "")
  #
  set(GD32_C_FLAGS
      "-DPICO_COPY_TO_RAM=1 ${GD32_SPECS}  ${PLATFORM_C_FLAGS} ${G32_DEBUG_FLAGS} -ffunction-sections -ggnu-pubnames --sysroot=${MINI_SYSROOT} -I${MINI_SYSROOT}/include  -DLN_ARCH=LN_ARCH_ARM   ${LN_BOARD_NAME_FLAG}  ${GD32_MCU}"
      CACHE INTERNAL "")
  set(CMAKE_C_FLAGS
      "${GD32_C_FLAGS}"
      CACHE INTERNAL "")
  set(CMAKE_ASM_FLAGS
      "${GD32_C_FLAGS}"
      CACHE INTERNAL "")
  set(CMAKE_CXX_FLAGS
      "${GD32_C_FLAGS} -std=gnu++11 -fno-rtti -fno-exceptions -fno-threadsafe-statics"
      CACHE INTERNAL "")
  #
  set(GD32_LD_FLAGS
      "   -nostdlib ${GD32_SPECS}  ${GD32_MCU}  ${GD32_LD_EXTRA}  ${GD32_LIBC}"
      CACHE INTERNAL "")
  set(GD32_LD_LIBS
      " -lgcc -Wl,--gc-sections  "
      CACHE INTERNAL "")
  #
  set(CLANG_LINKER_OPT
      ""
      CACHE INTERNAL "")
  #
  # --sysroot=${LN_CLANG_SYSROOT}
  #
  set(CMAKE_CXX_LINK_EXECUTABLE
      "<CMAKE_LINKER>  <CMAKE_CXX_LINK_FLAGS>  <LINK_FLAGS> ${LN_LTO}    -Wl,--start-group ${CRT} ${SB2} <OBJECTS>  <LINK_LIBRARIES>  -Wl,--end-group -lc  -Wl,-Map,<TARGET>.map   -o <TARGET> ${GD32_LD_FLAGS} ${GD32_LD_LIBS}  ${CLANG_LINKER_OPT} -e _entry_point"
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
  message(STATUS "Runime           ${PLATFORM_CLANG_C_FLAGS}")
endif()
