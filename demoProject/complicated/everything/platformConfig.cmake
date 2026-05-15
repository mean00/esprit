if(WIN32)
  set(PLATFORM_TOOLCHAIN_SUFFIX ".exe")
endif(WIN32)

if("${LN_ARCH}" STREQUAL "RISCV") # RISCV
  set(PLATFORM_PREFIX riscv32-unknown-elf-)
  set(PLATFORM_C_FLAGS "-march=rv32imac -mabi=ilp32 -mcmodel=medlow")
  if(WIN32)
    set(PLATFORM_TOOLCHAIN_PATH /c/gd32/toolchain/bin/) # Use /c/foo or c:\foo depending if you use mingw cmake or win32
                                                        # cmake
  else(WIN32)
    set(PLATFORM_TOOLCHAIN_PATH /riscv/tools/bin/)
  endif(WIN32)
else()
  set(PLATFORM_PREFIX arm-none-eabi-)
  set(PLATFORM_C_FLAGS " ")
  if(WIN32)
    set(PLATFORM_TOOLCHAIN_PATH "/c/dev/arm83/bin")
  else()
    # SET(PLATFORM_TOOLCHAIN_PATH "/home/fx/Arduino_stm32/arm-gcc-2020q2/bin")
    set(PLATFORM_TOOLCHAIN_PATH "/home/fx/Arduino_stm32/arm-gcc-2020q4/bin")
  endif()
endif()
