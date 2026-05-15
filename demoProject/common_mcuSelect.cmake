# We ony enable usb for arm and non small footprint
if(NOT DEFINED LN_ARCH)
  if(USE_CH32V3x) # RISCV
    set(LN_ENABLED_BOARDS
        CH32F1
        RP2040
        RP2350
        STM32F1
        NRF51
        NRF91
        LPC11XX
        LPC15XX
        LPC17XX
        STM32F4
        STM32G0
        STM32H7
        STM32H5
        STM32L4
        CACHE INTERNAL "")
    # try without crystal... SET(LN_USE_INTERNAL_CLOCK True        CACHE INTERNAL "") SET(LN_MCU_SPEED          48000000
    # CACHE INTERNAL "") # 96 Mhz SET(LN_MCU_SPEED          96000000    CACHE INTERNAL "") # 96 Mhz
    set(LN_MCU_SPEED
        144000000
        CACHE INTERNAL "") # 144 Mhz

    set(LN_ARCH
        "RISCV"
        CACHE INTERNAL "")
    set(LN_MCU
        "CH32V3x"
        CACHE INTERNAL "")
    set(LN_MCU_RAM_SIZE
        64
        CACHE INTERNAL "")
    set(LN_MCU_FLASH_SIZE
        256
        CACHE INTERNAL "")
    set(LN_MCU_STATIC_RAM
        10
        CACHE INTERNAL "")
    set(LN_SPEC
        "picolibc"
        CACHE INTERNAL "") # if not set we use nano
    set(LN_BOOTLOADER_SIZE
        0
        CACHE INTERNAL "")
    # SET(LN_SPEC         "picolibc"   CACHE INTERNAL "") # if not set we use nano
  else()
    if(USE_GD32F3)
      set(LN_ENABLED_BOARDS
          CH32F1
          RP2040
          RP2350
          STM32F1
          NRF51
          NRF91
          LPC11XX
          LPC15XX
          LPC17XX
          STM32F4
          STM32G0
          STM32H7
          STM32H5
          STM32L4
          CACHE INTERNAL "")
      set(LN_ARCH
          "ARM"
          CACHE INTERNAL "")
      set(LN_MCU
          "M4"
          CACHE INTERNAL "")
      set(LN_MCU_FLASH_SIZE
          256
          CACHE INTERNAL "")
      set(LN_MCU_RAM_SIZE
          46
          CACHE INTERNAL "")
      set(LN_MCU_STATIC_RAM
          8
          CACHE INTERNAL "")
      # SET(LN_MCU_SPEED 72000000    CACHE INTERNAL "") #=> ok
      set(LN_MCU_SPEED
          96000000
          CACHE INTERNAL "") # => ok

      # SET(LN_MCU_SPEED  72000000    CACHE INTERNAL "") #=> ok
      set(LN_SPEC
          "picolibc"
          CACHE INTERNAL "") # if not set we use nano
      set(LN_BOOTLOADER_SIZE
          0
          CACHE INTERNAL "")
    elseif(USE_RP2040)
      set(LN_ENABLED_BOARDS
          CH32F1
          RP2040
          RP2350
          STM32F1
          NRF51
          NRF91
          LPC11XX
          LPC15XX
          LPC17XX
          STM32F4
          STM32G0
          STM32H7
          STM32H5
          STM32L4
          CACHE INTERNAL "")
      set(LN_ARCH
          "ARM"
          CACHE INTERNAL "")
      set(LN_MCU
          "RP2040"
          CACHE INTERNAL "")
      if(USE_RP2040_PURE_RAM)
        set(LN_MCU_FLASH_SIZE
            128
            CACHE INTERNAL "")
        set(LN_MCU_RAM_SIZE
            64
            CACHE INTERNAL "")
      else()
        set(LN_MCU_RAM_SIZE
            200
            CACHE INTERNAL "")
        set(LN_MCU_FLASH_SIZE
            2048
            CACHE INTERNAL "")
      endif()
      set(LN_MCU_SPEED
          125000000
          CACHE INTERNAL "")
      set(LN_MCU_STATIC_RAM
          10
          CACHE INTERNAL "")
      set(LN_MCU_EEPROM_SIZE
          2
          CACHE INTERNAL "")
      set(LN_BOOTLOADER_SIZE
          0
          CACHE INTERNAL "")
      set(LN_SPEC
          "picolibc"
          CACHE INTERNAL "") # if not set we use nano

    elseif(USE_RP2350)
      set(LN_ENABLED_BOARDS
          CH32F1
          RP2040
          RP2350
          STM32F1
          NRF51
          NRF91
          LPC11XX
          LPC15XX
          LPC17XX
          STM32F4
          STM32G0
          STM32H7
          STM32H5
          STM32L4
          CACHE INTERNAL "")
      set(LN_ARCH
          "ARM"
          CACHE INTERNAL "")
      set(LN_MCU
          "RP2350"
          CACHE INTERNAL "")
      if(USE_RP2350_PURE_RAM)
        set(LN_MCU_FLASH_SIZE
            256
            CACHE INTERNAL "")
        set(LN_MCU_RAM_SIZE
            256
            CACHE INTERNAL "")
      else()
        set(LN_MCU_RAM_SIZE
            512
            CACHE INTERNAL "")
        set(LN_MCU_FLASH_SIZE
            2048
            CACHE INTERNAL "")
      endif()
      set(LN_MCU_SPEED
          150000000
          CACHE INTERNAL "")
      set(LN_MCU_STATIC_RAM
          10
          CACHE INTERNAL "")
      set(LN_MCU_EEPROM_SIZE
          2
          CACHE INTERNAL "")
      set(LN_BOOTLOADER_SIZE
          0
          CACHE INTERNAL "")
      set(LN_SPEC
          "picolibc"
          CACHE INTERNAL "") # if not set we use nano

    else() # Small bluepill style
      set(LN_ARCH
          "ARM"
          CACHE INTERNAL "")
      set(LN_MCU
          "M3"
          CACHE INTERNAL "")
      set(LN_MCU_RAM_SIZE
          20
          CACHE INTERNAL "")
      set(LN_MCU_SPEED
          72000000
          CACHE INTERNAL "")
      set(LN_ENABLED_BOARDS
          CH32F1 RP2040 RP2350 STM32F1
          CACHE INTERNAL "")
      if(USE_SMALLFOOTPRINT)
        set(LN_MCU_STATIC_RAM
            3
            CACHE INTERNAL "")
        set(LN_MCU_FLASH_SIZE
            64
            CACHE INTERNAL "")
        set(LN_MCU_EEPROM_SIZE
            2
            CACHE INTERNAL "")
      else() # "Big" flash
        set(LN_MCU_STATIC_RAM
            7
            CACHE INTERNAL "")
        set(LN_MCU_FLASH_SIZE
            188
            CACHE INTERNAL "")
        set(LN_MCU_EEPROM_SIZE
            2
            CACHE INTERNAL "")
        set(LN_BOOTLOADER_SIZE
            0
            CACHE INTERNAL "")
      endif()
      set(LN_SPEC
          "picolibc"
          CACHE INTERNAL "") # if not set we use nano
    endif() # GD32F3
  endif() # CH32V3
endif(NOT DEFINED LN_ARCH)
message(STATUS "Architecture ${LN_ARCH}, MCU=${LN_MCU}")
