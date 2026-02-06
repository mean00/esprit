#============================================================================= #
# https://interrupt.memfault.com/blog/arm-cortexm-with-llvm-clang
#============================================================================= #
MESSAGE(STATUS "Setting up GD32/arm cmake environment")
IF(NOT DEFINED LN_EXT)
  SET(LN_EXT arm_gd32fx CACHE INTERNAL "")
  SET(LN_TOOLCHAIN_EXT  arm_gd32fx_clang CACHE INTERNAL "")

  include(${ESPRIT_ROOT}/../platformConfig.cmake)

  IF(NOT PLATFORM_TOOLCHAIN_PATH)
    MESSAGE(FATAL_ERROR "PLATFORM_TOOLCHAIN_PATH is not defined in platformConfig.cmake !!")
  ENDIF(NOT PLATFORM_TOOLCHAIN_PATH)

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
  #
  # Setup toolchain for cross compilation
  #
  SET(CMAKE_SYSTEM_NAME Generic CACHE INTERNAL "")
  SET(CMAKE_C_COMPILER_ID   "Clang" CACHE INTERNAL "")
  SET(CMAKE_CXX_COMPILER_ID "Clang" CACHE INTERNAL "")
  set(CMAKE_C_COMPILER_WORKS      TRUE)
  set(CMAKE_CXX_COMPILER_WORKS    TRUE)
  #
  SET(LN_BOARD_NAME       bluepill CACHE INTERNAL "")

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
    set(CMAKE_C_COMPILER    ${PLATFORM_CLANG_PATH}/clang${PLATFORM_CLANG_VERSION}${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
    set(CMAKE_ASM_COMPILER  ${PLATFORM_CLANG_PATH}/clang${PLATFORM_CLANG_VERSION}${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
    set(CMAKE_CXX_COMPILER  ${PLATFORM_CLANG_PATH}/clang++${PLATFORM_CLANG_VERSION}${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
  ENDIF(USE_SCAN_BUILD)

  set(CMAKE_SIZE          ${PLATFORM_CLANG_PATH}/llvm-size${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
  set(CMAKE_OBJCOPY       ${PLATFORM_CLANG_PATH}/llvm-objcopy${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
  set(CMAKE_AR            ${PLATFORM_CLANG_PATH}/llvm-ar${TOOLCHAIN_SUFFIX}                           CACHE PATH "" FORCE)
  set(CMAKE_RANLIB        ${PLATFORM_CLANG_PATH}/llvm-ranlib${TOOLCHAIN_SUFFIX}                           CACHE PATH "" FORCE)

  # We use gcc linker, no we dont
  set(CMAKE_LINKER  ${CMAKE_CXX_COMPILER} CACHE PATH "" FORCE)

  # dont try to create a shared lib, it will not work
  SET(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

  MESSAGE(STATUS "GD32 C   compiler ${CMAKE_C_COMPILER}")
  MESSAGE(STATUS "GD32 C++ compiler ${CMAKE_CXX_COMPILER}")



  # M3 or M4 ?

  IF( "${LN_MCU}" STREQUAL "M3")
    SET(GD32_MCU "  -mfloat-abi=soft --target=armv7m-none-eabi  ")
    SET(MINI_SYSROOT "${PLATFORM_CLANG_PATH}/../lib/clang-runtimes/arm-none-eabi/" CACHE INTERNAL "")
    SET(GD32_LIBC "-L${MINI_SYSROOT}/lib " CACHE INTERNAL "")
  ELSE()
    IF( "${LN_MCU}" STREQUAL "M4")
      #SET(MINI_SYSROOT "${PLATFORM_CLANG_PATH}/../lib/clang-runtimes/arm-none-eabi/armv7m_hard_fpv4_sp_d16/" CACHE INTERNAL "")
      SET(GD32_LIBC "-L${MINI_SYSROOT}/lib " CACHE INTERNAL "")
      SET(GD32_MCU "-Wno-arm-interrupt-vfp-clobber -mcpu=cortex-m4 --target=armv7m-none-eabi   -mfloat-abi=hard -mfpu=fpv4-sp-d16  -DLN_USE_FPU=1")
    ELSE()
      MESSAGE(FATAL_ERROR "Unsupported MCU : only M3 is supported (works for M0+) : ${LN_MCU}")
    ENDIF()
  ENDIF()

  #____________________________
  # CLANG LINK USING GCC LIBS
  #____________________________

  SET(G32_DEBUG_FLAGS "-g3 ${LN_LTO}  -Oz  -gdwarf-4" CACHE INTERNAL "")
  SET(GD32_LD_EXTRA "  -Wl,--unresolved-symbols=report-all -Wl,--warn-common  " CACHE INTERNAL "")
  #
  SET(GD32_C_FLAGS  "${GD32_SPECS}  ${PLATFORM_C_FLAGS} ${G32_DEBUG_FLAGS} -ffunction-sections -ggnu-pubnames --sysroot=${MINI_SYSROOT} -I${MINI_SYSROOT}/include --target=arm-none-eabi -DLN_ARCH=LN_ARCH_ARM   ${LN_BOARD_NAME_FLAG}  ${GD32_MCU}" CACHE INTERNAL "")
  SET(CMAKE_C_FLAGS "${GD32_C_FLAGS}" CACHE INTERNAL "")
  SET(CMAKE_ASM_FLAGS "${GD32_C_FLAGS}" CACHE INTERNAL "")
  SET(CMAKE_CXX_FLAGS "${GD32_C_FLAGS} -std=gnu++11 -fno-rtti -fno-exceptions -fno-threadsafe-statics" CACHE INTERNAL "")
  #
  SET(GD32_LD_FLAGS " -fuse-ld=lld -nostdlib ${GD32_SPECS}  ${GD32_MCU}  ${GD32_LD_EXTRA}  ${GD32_LIBC}" CACHE INTERNAL "")
  SET(GD32_LD_LIBS "-lm  -Wl,--gc-sections -Wl,--gdb-index " CACHE INTERNAL "")
  #
  #SET(CLANG_LINKER_OPT "${MINI_SYSROOT}/lib/libclang_rt.builtins.a")
  #
  set(CMAKE_CXX_LINK_EXECUTABLE    "<CMAKE_LINKER>  <CMAKE_CXX_LINK_FLAGS>  <LINK_FLAGS> ${LN_LTO} --sysroot=${MINI_SYSROOT}  -Wl,--cref   -Wl,--start-group  <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group  -Wl,-Map,<TARGET>.map   -o <TARGET> ${GD32_LD_FLAGS} ${GD32_LD_LIBS} ${CLANG_LINKER_OPT}" CACHE INTERNAL "")
  SET(CMAKE_EXECUTABLE_SUFFIX_C .elf CACHE INTERNAL "")
  SET(CMAKE_EXECUTABLE_SUFFIX_CXX .elf CACHE INTERNAL "")


  MESSAGE(STATUS "MCU Architecture ${LN_ARCH}")
  MESSAGE(STATUS "MCU Type         ${LN_MCU}")
  MESSAGE(STATUS "MCU Ext          ${LN_EXT}")
  MESSAGE(STATUS "MCU Speed        ${LN_MCU_SPEED}")
  MESSAGE(STATUS "MCU Flash Size   ${LN_MCU_FLASH_SIZE}")
  MESSAGE(STATUS "MCU Ram Size     ${LN_MCU_RAM_SIZE}")
  MESSAGE(STATUS "MCU Static RAM   ${LN_MCU_STATIC_RAM}")
  MESSAGE(STATUS "RunTime          ${PLATFORM_CLANG_C_FLAGS}")
  MESSAGE(STATUS "Boards           ${LN_BOARD_NAME}")
ENDIF(NOT DEFINED LN_EXT)
