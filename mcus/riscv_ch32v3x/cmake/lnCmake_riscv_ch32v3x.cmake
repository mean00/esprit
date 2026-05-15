include(ln_merge_libs)

macro(GENERATE_GD32_FIRMWARE target)
  ln_merge_libs()

  if(NOT LN_MCU_RAM_SIZE)
    set(LN_MCU_RAM_SIZE
        32
        CACHE INTERNAL "")
  endif(NOT LN_MCU_RAM_SIZE)
  if(NOT LN_MCU_FLASH_SIZE)
    set(LN_MCU_FLASH_SIZE
        128
        CACHE INTERNAL "")
  endif(NOT LN_MCU_FLASH_SIZE)
  if(NOT LN_MCU_EEPROM_SIZE)
    set(LN_MCU_EEPROM_SIZE
        4
        CACHE INTERNAL "")
  endif(NOT LN_MCU_EEPROM_SIZE)

  if(USE_CH32v3x_PURE_RAM)
    configure_file("${LN_MCU_FOLDER}/boards/ch32v3x/ld_ram.lds.in" "${CMAKE_BINARY_DIR}/linker_script.ld" @ONLY)
  else()
    configure_file("${LN_MCU_FOLDER}/boards/ch32v3x/ld.lds.in" "${CMAKE_BINARY_DIR}/linker_script.ld" @ONLY)
  endif()

  add_executable(${target} ${LN_MCU_FOLDER}/start.S ${ARGN})
  target_link_libraries(${target} PUBLIC esprit_single_lib esprit_dev) # duplicates are NOT a mistake !
  target_link_options(${target} PRIVATE "-T${CMAKE_BINARY_DIR}/linker_script.ld")
  target_link_directories(${target} PUBLIC ${CMAKE_BINARY_DIR})
  add_custom_command(
    TARGET ${target}
    POST_BUILD
    COMMAND ${CMAKE_OBJCOPY} -Obinary $<TARGET_FILE:${target}> $<TARGET_FILE:${target}>.bin
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Generating bin file")
endmacro()
macro(HASH_GD32_FIRMWARE target)
  add_custom_command(
    TARGET ${target}
    POST_BUILD
    COMMAND python3 ${ESPRIT_ROOT}/script/lnCRC32Checksum.py $<TARGET_FILE:${target}>.bin
            $<TARGET_FILE:${target}>.ck_bin
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Generating checksumed file")
endmacro()
include(ln_use_library)
