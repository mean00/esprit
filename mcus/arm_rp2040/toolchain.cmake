#=============================================================================#
# https://interrupt.memfault.com/blog/arm-cortexm-with-llvm-clang
MESSAGE(STATUS "Setting up RP2040/arm cmake environment")
IF(NOT DEFINED LN_EXT)
  SET(LN_EXT arm_rp2040 CACHE INTERNAL "")
  SET(LN_TOOLCHAIN_EXT  arm_rp2040_clang CACHE INTERNAL "")

  INCLUDE(${ESPRIT_ROOT}/../platformConfig.cmake)

  IF(NOT PLATFORM_TOOLCHAIN_PATH)
    MESSAGE(FATAL_ERROR "PLATFORM_TOOLCHAIN_PATH is not defined in platformConfig.cmake !!")
  ENDIF()
  #
  #SET(LN_LTO "-flto")
  #
  LIST(APPEND CMAKE_SYSTEM_PREFIX_PATH "${PLATFORM_TOOLCHAIN_PATH}")

  FUNCTION(FATAL_BANNER msg)
    MESSAGE(STATUS "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@")
    MESSAGE(STATUS "${msg}")
    MESSAGE(STATUS "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@")
    MESSAGE(FATAL_ERROR "${msg}")
  ENDFUNCTION()

  #
  # Sanity check
  #
  #
  # Setup toolchain for cross compilation
  #
  SET(CMAKE_SYSTEM_NAME Generic CACHE INTERNAL "")
  SET(CMAKE_C_COMPILER_ID   "GNU" CACHE INTERNAL "")
  SET(CMAKE_CXX_COMPILER_ID "GNU" CACHE INTERNAL "")
  SET(CMAKE_C_COMPILER_WORKS      TRUE)
  SET(CMAKE_CXX_COMPILER_WORKS    TRUE)
  #
  SET(LN_BOARD_NAME       rp2040 CACHE INTERNAL "")

  # Speed
  IF(USE_SCAN_BUILD)
  ELSE()
    set(CMAKE_C_COMPILER   ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}gcc${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
    set(CMAKE_ASM_COMPILER ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}gcc${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
    set(CMAKE_CXX_COMPILER ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}g++${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
  ENDIF()

  SET(CMAKE_SIZE      ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}size${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
  SET(CMAKE_OBJCOPY   ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}objcopy${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
  SET(CMAKE_OBJDUMP   ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}objdump${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
  SET(CMAKE_AR        ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}ar${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)
  SET(CMAKE_RANLIB    ${PLATFORM_TOOLCHAIN_PATH}/${PLATFORM_PREFIX}ranlib${TOOLCHAIN_SUFFIX} CACHE PATH "" FORCE)

  # We use gcc linker, no we dont
  SET(CMAKE_LINKER  ${CMAKE_CXX_COMPILER} CACHE PATH "" FORCE)

  # dont try to create a shared lib, it will not work
  SET(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)


  # OK, now setup the pico sdk
  #---------------------------------------------------------
  SET(PICO_COMPILER  pico_arm_clang CACHE INTERNAL "")
  SET(PICO_TOOLCHAIN_PATH  /arm/tools_llvm/bin CACHE INTERNAL "")
  SET(LD_SCRIPT ${CMAKE_BINARY_DIR}/linker_script.ld CACHE INTERNAL "")
  SET(PICO_BOARD "pico")
  SET(PICO_COMPILER  pico_arm_clang CACHE INTERNAL "")
  SET(PICO_TOOLCHAIN_PATH  ${PLATFORM_CLANG_PATH} CACHE INTERNAL "")

  #---------------------------------------------------------


  IF(NOT DEFINED LN_MCU_SPEED)
    SET(LN_MCU_SPEED 125000000)
  ENDIF()

  # Set default value
  IF(NOT LN_MCU_RAM_SIZE)
    MESSAGE(STATUS "Ram size not set, using default")
    SET(LN_MCU_RAM_SIZE 230)
  ENDIF()
  IF(NOT LN_MCU_FLASH_SIZE)
    MESSAGE(STATUS "Flash size not set, using default")
    SET(LN_MCU_FLASH_SIZE 4000)
  ENDIF()
  IF(NOT LN_MCU_EEPROM_SIZE)
    MESSAGE(STATUS "NVME size not set, using default")
    SET(LN_MCU_EEPROM_SIZE 4)
  ENDIF()
  IF(NOT LN_MCU_STATIC_RAM)
    MESSAGE(STATUS "Static ram size not set, using default")
    SET(LN_MCU_STATIC_RAM 6)
  ENDIF()
  #
  SET(LN_MCU_RAM_SIZE ${LN_MCU_RAM_SIZE} CACHE INTERNAL "" FORCE)
  SET(LN_MCU_FLASH_SIZE ${LN_MCU_FLASH_SIZE} CACHE INTERNAL "" FORCE)
  SET(LN_MCU_EEPROM_SIZE ${LN_MCU_EEPROM_SIZE} CACHE INTERNAL "" FORCE)
  SET(LN_MCU_SPEED ${LN_MCU_SPEED} CACHE INTERNAL "" FORCE)
  SET(LN_MCU_STATIC_RAM ${LN_MCU_STATIC_RAM} CACHE INTERNAL "" FORCE)

  MESSAGE(STATUS "GD32 C   compiler ${CMAKE_C_COMPILER}")
  MESSAGE(STATUS "GD32 C++ compiler ${CMAKE_CXX_COMPILER}")



  # M0+

  SET(GD32_LIBC "-L${MINI_SYSROOT}/lib " CACHE INTERNAL "")
  SET(GD32_MCU "-mcpu=cortex-m0plus  -DUSE_RP2040  -DPICO_RP2040_USB_FAST_IRQ=0  " CACHE INTERNAL "")

  #____________________________
  # CLANG LINK USING GCC LIBS
  #____________________________
  #SET(LN_CLANG_LIBC /home/fx/Arduino_stm32/arm-gcc-2020q4/bin/../lib/gcc/arm-none-eabi/10.2.1/thumb/v7-m/nofp/libgcc.a CACHE INTERNAL "")
  #SET(LN_CLANG_MULTILIB thumb/v7-m/nofp CACHE INTERNAL "")

  #SET(G32_DEBUG_FLAGS "-g3 ${LN_LTO}  -O0  -gdwarf-4" CACHE INTERNAL "")
  SET(G32_DEBUG_FLAGS "-g3 ${LN_LTO}  -O1 -gdwarf-4" CACHE INTERNAL "")

  SET(GD32_LD_EXTRA "  -Wl,--unresolved-symbols=report-all -Wl,--warn-common  " CACHE INTERNAL "")
  #
  SET(GD32_C_FLAGS  "-DPICO_COPY_TO_RAM=1 ${GD32_SPECS}  ${PLATFORM_C_FLAGS} ${G32_DEBUG_FLAGS} -ffunction-sections -ggnu-pubnames --sysroot=${MINI_SYSROOT} -I${MINI_SYSROOT}/include  -DLN_ARCH=LN_ARCH_ARM   ${LN_BOARD_NAME_FLAG}  ${GD32_MCU}" CACHE INTERNAL "")
  SET(CMAKE_C_FLAGS "${GD32_C_FLAGS}" CACHE INTERNAL "")
  SET(CMAKE_ASM_FLAGS "${GD32_C_FLAGS}" CACHE INTERNAL "")
  SET(CMAKE_CXX_FLAGS "${GD32_C_FLAGS} -std=gnu++11 -fno-rtti -fno-exceptions -fno-threadsafe-statics" CACHE INTERNAL "")
  #
  SET(GD32_LD_FLAGS "   -nostdlib ${GD32_SPECS}  ${GD32_MCU}  ${GD32_LD_EXTRA}  ${GD32_LIBC}" CACHE INTERNAL "")
  SET(GD32_LD_LIBS " -lgcc -Wl,--gc-sections  " CACHE INTERNAL "")
  #
  SET(CLANG_LINKER_OPT "" CACHE INTERNAL "")
  #
  # --sysroot=${LN_CLANG_SYSROOT}
  #
  #
  SET(CMAKE_CXX_LINK_EXECUTABLE    "<CMAKE_LINKER>  <CMAKE_CXX_LINK_FLAGS>  <LINK_FLAGS> ${LN_LTO}    -Wl,--start-group ${CRT} ${SB2} <OBJECTS>  <LINK_LIBRARIES>  -Wl,--end-group -lc  -Wl,-Map,<TARGET>.map   -o <TARGET> ${GD32_LD_FLAGS} ${GD32_LD_LIBS}  ${CLANG_LINKER_OPT} -e _entry_point" CACHE INTERNAL "")
  SET(CMAKE_EXECUTABLE_SUFFIX_C .elf CACHE INTERNAL "")
  SET(CMAKE_EXECUTABLE_SUFFIX_CXX .elf CACHE INTERNAL "")






  MESSAGE(STATUS "MCU Architecture ${LN_ARCH}")
  MESSAGE(STATUS "MCU Type         ${LN_MCU}")
  MESSAGE(STATUS "MCU Speed        ${LN_MCU_SPEED}")
  MESSAGE(STATUS "MCU Flash Size   ${LN_MCU_FLASH_SIZE}")
  MESSAGE(STATUS "MCU Ram Size     ${LN_MCU_RAM_SIZE}")
  MESSAGE(STATUS "MCU Static RAM   ${LN_MCU_STATIC_RAM}")
  MESSAGE(STATUS "Runime           ${PLATFORM_CLANG_C_FLAGS}")
ENDIF()
