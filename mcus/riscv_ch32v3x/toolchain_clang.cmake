#=============================================================================#
MESSAGE(STATUS "Setting up CH32V3x riscv cmake environment")
IF(NOT DEFINED LN_EXT)
  SET(LN_EXT riscv_ch32v3x            CACHE INTERNAL "")
  include(${ESPRIT_ROOT}/../platformConfig.cmake)
  SET(LN_TOOLCHAIN_EXT  riscv_ch32v3x_clang CACHE INTERNAL "")

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
    #FATAL_BANNER( "!! PLATFORM_TOOLCHAIN_PATH does not point to a valid toolchain ( ${PLATFORM_PREFIX}gcc....)!! (${PLATFORM_TOOLCHAIN_PATH})")
  ENDIF()
  #
  # Setup toolchain for cross compilation
  #
  SET(CMAKE_SYSTEM_NAME Generic           CACHE INTERNAL "")
  SET(CMAKE_C_COMPILER_ID   "Clang"       CACHE INTERNAL "")
  SET(CMAKE_CXX_COMPILER_ID "Clang"       CACHE INTERNAL "")
  set(CMAKE_C_COMPILER_WORKS      TRUE    CACHE INTERNAL "")
  set(CMAKE_CXX_COMPILER_WORKS    TRUE    CACHE INTERNAL "")

  #
  SET(LN_BOARD_NAME_FLAG    ""       CACHE INTERNAL "")
  SET(LN_BOARD_NAME       ch32v3x    CACHE INTERNAL "")

  IF(NOT DEFINED LN_MCU_SPEED)
    SET(LN_MCU_SPEED 72000000   CACHE INTERNAL "")
  ENDIF()
  # Set default value
  IF(NOT LN_MCU_RAM_SIZE)
    MESSAGE(STATUS "Ram size not set, using default")
    SET(LN_MCU_RAM_SIZE 64      CACHE INTERNAL "")
  ENDIF(NOT LN_MCU_RAM_SIZE)
  IF(NOT LN_MCU_FLASH_SIZE        )
    MESSAGE(STATUS "Flash size not set, using default")
    SET(LN_MCU_FLASH_SIZE 256   CACHE INTERNAL "")
  ENDIF(NOT LN_MCU_FLASH_SIZE)
  IF(NOT LN_MCU_EEPROM_SIZE)
    MESSAGE(STATUS "NVME size not set, using default")
    SET(LN_MCU_EEPROM_SIZE 4    CACHE INTERNAL "")
  ENDIF(NOT LN_MCU_EEPROM_SIZE)
  IF(NOT LN_MCU_STATIC_RAM)
    MESSAGE(STATUS "Static ram size not set, using default")
    SET(LN_MCU_STATIC_RAM 6     CACHE INTERNAL "")
  ENDIF(NOT LN_MCU_STATIC_RAM)
  IF(NOT LN_MCU_XTAL_CLOCK)
    MESSAGE(STATUS "Xtal clock not set,  using default (8 Mhz)")
    SET(LN_MCU_XTAL_CLOCK 8     CACHE INTERNAL "")
  ENDIF()


  #
  SET(LN_MCU_RAM_SIZE ${LN_MCU_RAM_SIZE}          CACHE INTERNAL "" FORCE)
  SET(LN_MCU_FLASH_SIZE ${LN_MCU_FLASH_SIZE}      CACHE INTERNAL "" FORCE)
  SET(LN_MCU_EEPROM_SIZE ${LN_MCU_EEPROM_SIZE}    CACHE INTERNAL "" FORCE)
  SET(LN_MCU_SPEED ${LN_MCU_SPEED}                CACHE INTERNAL "" FORCE)

  #
  IF(USE_SCAN_BUILD)
  ELSE(USE_SCAN_BUILD)
    set(CMAKE_C_COMPILER    ${PLATFORM_CLANG_PATH}/clang${PLATFORM_CLANG_VERSION}${TOOLCHAIN_SUFFIX}    CACHE PATH "" FORCE)
    set(CMAKE_ASM_COMPILER  ${PLATFORM_CLANG_PATH}/clang${PLATFORM_CLANG_VERSION}${TOOLCHAIN_SUFFIX}    CACHE PATH "" FORCE)
    set(CMAKE_CXX_COMPILER  ${PLATFORM_CLANG_PATH}/clang++${PLATFORM_CLANG_VERSION}${TOOLCHAIN_SUFFIX}  CACHE PATH "" FORCE)
  ENDIF(USE_SCAN_BUILD)
  set(CMAKE_SIZE          ${PLATFORM_CLANG_PATH}/llvm-size${TOOLCHAIN_SUFFIX}                         CACHE PATH "" FORCE)
  set(CMAKE_AR            ${PLATFORM_CLANG_PATH}/llvm-ar${TOOLCHAIN_SUFFIX}                           CACHE PATH "" FORCE)
  set(CMAKE_RANLIB        ${PLATFORM_CLANG_PATH}/llvm-ranlib${TOOLCHAIN_SUFFIX}                           CACHE PATH "" FORCE)
  set(CMAKE_OBJCOPY       ${PLATFORM_CLANG_PATH}/llvm-objcopy${TOOLCHAIN_SUFFIX}                      CACHE PATH "" FORCE)
  #
  #set(CMAKE_LINKER  ${CMAKE_CXX_COMPILER}                                                             CACHE PATH "" FORCE)

  # dont try to create a shared lib, it will not work
  SET(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY                                                    CACHE INTERNAL "")

  MESSAGE(STATUS "GD32 C   compiler ${CMAKE_C_COMPILER}")
  MESSAGE(STATUS "GD32 C++ compiler ${CMAKE_CXX_COMPILER}")
  IF(LN_SPEC)
    SET(LN_SPEC "${LN_SPEC}"                    CACHE INTERNAL "" FORCE)
  ELSE(LN_SPEC)
    SET(LN_SPEC "nano"                          CACHE INTERNAL "" FORCE)
  ENDIF(LN_SPEC)
  SET(GD32_SPECS  "--specs=${LN_SPEC}.specs"      CACHE INTERNAL "" FORCE)
  #
  SET(GD32_DEBUG_FLAGS "-g3 -gdwarf-5 ${LN_LTO} -Oz" CACHE INTERNAL "")
  #
  SET(GD32_MCU_C_FLAGS "--sysroot ${PLATFORM_CLANG_SYSROOT} ${PLATFORM_CLANG_C_FLAGS} -DLN_MCU=LN_MCU_CH32V3x -DLN_ARCH=LN_ARCH_RISCV ${LN_BOARD_NAME_FLAG} -I${ESPRIT_ROOT}/riscv_ch32v3x/" CACHE INTERNAL "" )
  SET(GD32_C_FLAGS    "-DLN_MCU_XTAL_CLOCK=${LN_MCU_XTAL_CLOCK} ${GD32_SPECS_SPECS} ${GD32_MCU_C_FLAGS} ${GD32_DEBUG_FLAGS}  -Werror=return-type  -fmessage-length=0 -fsigned-char -ffunction-sections -fdata-sections -fno-common " CACHE INTERNAL "")
  SET(CMAKE_C_FLAGS   "${GD32_C_FLAGS}"                                                       CACHE INTERNAL "")
  SET(CMAKE_ASM_FLAGS "${GD32_C_FLAGS}"                                                       CACHE INTERNAL "")
  SET(CMAKE_CXX_FLAGS "${GD32_C_FLAGS}  -fno-rtti -fno-exceptions -fno-threadsafe-statics"    CACHE INTERNAL "")
  #
  SET(CLANG_LINKER_OPT "" CACHE INTERNAL "")
  #
  SET(GD32_LD_FLAGS "-fuse-ld=lld  -L${PLATFORM_CLANG_SYSROOT}/lib/riscv32-unknown-unknown-elf ${LN_LTO} -nostdlib ${GD32_SPECS_SPECS} --sysroot ${PLATFORM_CLANG_SYSROOT}  -Wl,--warn-common" CACHE INTERNAL "")
  SET(GD32_LD_LIBS "-lm   ${CLANG_LINKER_OPT} -Wl,--gc-sections -Wl,--gdb-index " CACHE INTERNAL "")

  #
  set(CMAKE_CXX_LINK_EXECUTABLE    "<CMAKE_CXX_COMPILER>  ${PLATFORM_CLANG_EXTRA_LD_ARG} <CMAKE_CXX_LINK_FLAGS>  ${PLATFORM_CLANG_C_FLAGS} <LINK_FLAGS>   -Wl,--start-group  <OBJECTS> <LINK_LIBRARIES> -Wl,--end-group  -Wl,-Map,<TARGET>.map   -o <TARGET> ${GD32_LD_FLAGS} ${GD32_LD_LIBS} ${GD32_LD_FLAGS}   -lclang_rt.builtins "  CACHE INTERNAL "")
  SET(CMAKE_EXECUTABLE_SUFFIX_C .elf                                                                  CACHE INTERNAL "")
  SET(CMAKE_EXECUTABLE_SUFFIX_CXX .elf                                                                CACHE INTERNAL "")

  MESSAGE(STATUS "MCU Architecture ${LN_ARCH}")
  MESSAGE(STATUS "MCU Type         ${LN_MCU}")
  MESSAGE(STATUS "MCU Speed        ${LN_MCU_SPEED}")
  MESSAGE(STATUS "MCU Flash Size   ${LN_MCU_FLASH_SIZE}")
  MESSAGE(STATUS "MCU Ram Size     ${LN_MCU_RAM_SIZE}")
  MESSAGE(STATUS "MCU Static RAM   ${LN_MCU_STATIC_RAM}")
  MESSAGE(STATUS "MCU Crystal clock ${LN_MCU_XTAL_CLOCK}")
  MESSAGE(STATUS "CFlags           ${PLATFORM_CLANG_C_FLAGS}")

ENDIF(NOT DEFINED LN_EXT)
