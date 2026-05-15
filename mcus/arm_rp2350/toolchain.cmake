# =============================================================================#
# https://interrupt.memfault.com/blog/arm-cortexm-with-llvm-clang
message(STATUS "Setting up RP2350/arm cmake environment")
if(NOT DEFINED LN_EXT)
  set(LN_EXT
      arm_rp2350
      CACHE INTERNAL "")
  set(LN_TOOLCHAIN_EXT
      arm_rp2350_clang
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
  endfunction(FATAL_BANNER msg)

  #
  # Sanity check
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
  set(CMAKE_C_COMPILER_WORKS TRUE)
  set(CMAKE_CXX_COMPILER_WORKS TRUE)
  #
  set(LN_BOARD_NAME
      rp2350
      CACHE INTERNAL "")

  # Speed
  #
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

  #
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
  set(PICO_BOARD "sparkfun_promicro_rp2350")
  set(PICO_COMPILER
      pico_arm_clang
      CACHE INTERNAL "")
  set(PICO_TOOLCHAIN_PATH
      ${PLATFORM_CLANG_PATH}
      CACHE INTERNAL "")

  # ---------------------------------------------------------

  if(NOT DEFINED LN_MCU_SPEED)
    set(LN_MCU_SPEED 150000000)
  endif()

  # Set default value
  if(NOT LN_MCU_RAM_SIZE)
    message(STATUS "Ram size not set, using default")
    set(LN_MCU_RAM_SIZE 512)
  endif()
  if(NOT LN_MCU_FLASH_SIZE)
    message(STATUS "Flash size not set, using default")
    set(LN_MCU_FLASH_SIZE 4096)
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

  # This does not seem to work well...
  function(APX VAR NM VAL)
    set(VAR
        "${VAR} -D${NM}=\"${VAL}\""
        PARENT_SCOPE)
  endfunction()

  # M0+

  # SET(MINI_SYSROOT "${PLATFORM_CLANG_PATH}/../lib/clang-runtimes/arm-none-eabi/armv8m.main_hard_fp/" CACHE INTERNAL
  # "") SET(GD32_LIBC "-L${MINI_SYSROOT}/lib " CACHE INTERNAL "")
  set(GD32_MCU
      "-mcpu=cortex-m33 -DPICO_RP2350=1  -DPICO_PLATFORM=rp2350 -DUSE_RP2350 -DPICO_RP2350_USB_FAST_IRQ=0  "
      CACHE INTERNAL "")
  apx(GD32_MCU PICO_BOARD "${PICO_BOARD}")

  # ____________________________ CLANG LINK USING GCC LIBS ____________________________
  set(G32_DEBUG_FLAGS
      "-g3 ${LN_LTO}  -O1 -gdwarf-4"
      CACHE INTERNAL "")

  set(GD32_LD_EXTRA
      "  -Wl,--unresolved-symbols=report-all -Wl,--warn-common  "
      CACHE INTERNAL "")
  # SET(GD32_RP_FLAGS "-DLIB_BOOT_STAGE2_HEADERS=1 -DLIB_PICO_ATOMIC=1 -DLIB_PICO_BIT_OPS=1 -DLIB_PICO_BIT_OPS_PICO=1
  # -DLIB_PICO_CLIB_INTERFACE=1 -DLIB_PICO_CRT0=1 -DLIB_PICO_CXX_OPTIONS=1 -DLIB_PICO_DIVIDER=1
  # -DLIB_PICO_DIVIDER_COMPILER=1 -DLIB_PICO_DOUBLE=1 -DLIB_PICO_DOUBLE_PICO=1 -DLIB_PICO_FLASH=1 -DLIB_PICO_FLOAT=1
  # -DLIB_PICO_FLOAT_PICO=1 -DLIB_PICO_FLOAT_PICO_VFP=1 -DLIB_PICO_INT64_OPS=1 -DLIB_PICO_INT64_OPS_COMPILER=1
  # -DLIB_PICO_MALLOC=1 -DLIB_PICO_MEM_OPS=1 -DLIB_PICO_MEM_OPS_COMPILER=1 -DLIB_PICO_NEWLIB_INTERFACE=1
  # -DLIB_PICO_PLATFORM=1 -DLIB_PICO_PLATFORM_COMPILER=1 -DLIB_PICO_PLATFORM_PANIC=1 -DLIB_PICO_PLATFORM_SECTIONS=1
  # -DLIB_PICO_PRINTF=1 -DLIB_PICO_PRINTF_PICO=1 -DLIB_PICO_RUNTIME=1 -DLIB_PICO_RUNTIME_INIT=1
  # -DLIB_PICO_STANDARD_BINARY_INFO=1 -DLIB_PICO_STANDARD_LINK=1 -DLIB_PICO_STDIO=1 -DLIB_PICO_STDLIB=1
  # -DLIB_PICO_SYNC=1 -DLIB_PICO_SYNC_CRITICAL_SECTION=1 -DLIB_PICO_SYNC_MUTEX=1 -DLIB_PICO_SYNC_SEM=1 -DLIB_PICO_TIME=1
  # -DLIB_PICO_TIME_ADAPTER=1 -DLIB_PICO_UTIL=1 -DPICO_32BIT=1 -DPICO_BUILD=1  -DPICO_COPY_TO_RAM=0
  # -DPICO_CXX_ENABLE_EXCEPTIONS=0 -DPICO_NO_FLASH=0 -DPICO_NO_HARDWARE=0 -DPICO_ON_DEVICE=1 -DPICO_RP2350=1
  # -DPICO_USE_BLOCKED_RAM=0 ")
  set(GD32_RP_FLAGS "-include ${ESPRIT_ROOT}/mcus/arm_rp2350/include/pico_flags.h")
  # DLIB_BOOT_STAGE2_HEADERS=1 -DLIB_PICO_ATOMIC=1 -DLIB_PICO_BIT_OPS=1 -DLIB_PICO_BIT_OPS_PICO=1
  # -DLIB_PICO_CLIB_INTERFACE=1 -DLIB_PICO_CRT0=1 -DLIB_PICO_CXX_OPTIONS=1 -DLIB_PICO_DIVIDER=1
  # -DLIB_PICO_DIVIDER_COMPILER=1 -DLIB_PICO_DOUBLE=1 -DLIB_PICO_DOUBLE_PICO=1 -DLIB_PICO_FLASH=1 -DLIB_PICO_FLOAT=1
  # -DLIB_PICO_FLOAT_PICO=1 -DLIB_PICO_FLOAT_PICO_VFP=1 -DLIB_PICO_INT64_OPS=1 -DLIB_PICO_INT64_OPS_COMPILER=1
  # -DLIB_PICO_MALLOC=1 -DLIB_PICO_MEM_OPS=1 -DLIB_PICO_MEM_OPS_COMPILER=1 -DLIB_PICO_NEWLIB_INTERFACE=1
  # -DLIB_PICO_PLATFORM=1 -DLIB_PICO_PLATFORM_COMPILER=1 -DLIB_PICO_PLATFORM_PANIC=1 -DLIB_PICO_PLATFORM_SECTIONS=1
  # -DLIB_PICO_PRINTF=1 -DLIB_PICO_PRINTF_PICO=1 -DLIB_PICO_RUNTIME=1 -DLIB_PICO_RUNTIME_INIT=1
  # -DLIB_PICO_STANDARD_BINARY_INFO=1 -DLIB_PICO_STANDARD_LINK=1 -DLIB_PICO_STDIO=1 -DLIB_PICO_STDLIB=1
  # -DLIB_PICO_SYNC=1 -DLIB_PICO_SYNC_CRITICAL_SECTION=1 -DLIB_PICO_SYNC_MUTEX=1 -DLIB_PICO_SYNC_SEM=1 -DLIB_PICO_TIME=1
  # -DLIB_PICO_TIME_ADAPTER=1 -DLIB_PICO_UTIL=1 -DPICO_32BIT=1 -DPICO_BUILD=1  -DPICO_COPY_TO_RAM=0
  # -DPICO_CXX_ENABLE_EXCEPTIONS=0 -DPICO_NO_FLASH=0 -DPICO_NO_HARDWARE=0 -DPICO_ON_DEVICE=1 -DPICO_RP2350=1
  # -DPICO_USE_BLOCKED_RAM=0 ")
  set(GD32_RP_FLAGS "${GD32_RP_FLAGS} -D__force_inline='__attribute__((always_inline))'")
  apx(GD32_RP_FLAGS PICO_PROGRAM_NAME "code")
  apx(GD32_RP_FLAGS PICO_PROGRAM_VERSION_STRING "0.1")
  #
  # SET(GD32_C_FLAGS  "-DPICO_COPY_TO_RAM=1 ${GD32_SPECS}  ${PLATFORM_C_FLAGS} ${G32_DEBUG_FLAGS} -ffunction-sections
  # -ggnu-pubnames --sysroot=${MINI_SYSROOT} -I${MINI_SYSROOT}/include --target=arm-none-eabi -DLN_ARCH=LN_ARCH_ARM
  # ${LN_BOARD_NAME_FLAG}  ${GD32_MCU}" CACHE INTERNAL "")
  set(GD32_C_FLAGS
      " ${GD32_SPECS}  ${GD32_RP_FLAGS} ${PLATFORM_C_FLAGS} ${G32_DEBUG_FLAGS} -ffunction-sections -ggnu-pubnames --sysroot=${MINI_SYSROOT} -I${MINI_SYSROOT}/include  -DLN_ARCH=LN_ARCH_ARM   ${LN_BOARD_NAME_FLAG}  ${GD32_MCU}"
      CACHE INTERNAL "")
  set(CMAKE_C_FLAGS
      "${GD32_C_FLAGS}"
      CACHE INTERNAL "")
  set(CMAKE_ASM_FLAGS
      "${GD32_C_FLAGS} -Wno-unused-command-line-argument"
      CACHE INTERNAL "")
  set(CMAKE_CXX_FLAGS
      "${GD32_C_FLAGS} -std=gnu++11 -fno-rtti -fno-exceptions -fno-threadsafe-statics"
      CACHE INTERNAL "")
  #
  set(GD32_LD_FLAGS
      "   -lgcc -nostdlib ${GD32_SPECS}  ${GD32_MCU}  ${GD32_LD_EXTRA}  ${GD32_LIBC}"
      CACHE INTERNAL "")
  set(GD32_LD_LIBS
      " -Wl,--gc-sections  "
      CACHE INTERNAL "")
  #
  set(CLANG_LINKER_OPT "")
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
  message(STATUS "Runtime          ${PLATFORM_CLANG_C_FLAGS}")
endif()
