include(ln_merge_libs)

MACRO(GENERATE_GD32_FIRMWARE target)
  LN_MERGE_LIBS()

  IF(NOT LN_MCU_RAM_SIZE)
    SET(LN_MCU_RAM_SIZE 32 CACHE INTERNAL "")
  ENDIF(NOT LN_MCU_RAM_SIZE)
  IF(NOT LN_MCU_FLASH_SIZE)
    SET(LN_MCU_FLASH_SIZE 128 CACHE INTERNAL "")
  ENDIF(NOT LN_MCU_FLASH_SIZE)
  IF(NOT LN_MCU_EEPROM_SIZE)
    SET(LN_MCU_EEPROM_SIZE 4 CACHE INTERNAL "")
  ENDIF(NOT LN_MCU_EEPROM_SIZE)

  IF(USE_CH32v3x_PURE_RAM)
    configure_file( "${LN_MCU_FOLDER}/boards/ch32v3x/ld_ram.lds.in" "${CMAKE_BINARY_DIR}/linker_script.ld" @ONLY)
  ELSE()
    configure_file( "${LN_MCU_FOLDER}/boards/ch32v3x/ld.lds.in" "${CMAKE_BINARY_DIR}/linker_script.ld" @ONLY)
  ENDIF()

  ADD_EXECUTABLE(${target}  ${LN_MCU_FOLDER}/start.S ${ARGN})
  TARGET_LINK_LIBRARIES(${target} PUBLIC  esprit_single_lib esprit_dev ) # duplicates are NOT a mistake !
  TARGET_LINK_OPTIONS(${target}  PRIVATE "-T${CMAKE_BINARY_DIR}/linker_script.ld" )
  target_link_directories(${target} PUBLIC ${CMAKE_BINARY_DIR})
  add_custom_command(TARGET ${target}
        POST_BUILD
        COMMAND ${CMAKE_OBJCOPY} -Obinary $<TARGET_FILE:${target}> $<TARGET_FILE:${target}>.bin
        WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
        COMMENT "Generating bin file"
    )
ENDMACRO()
MACRO(HASH_GD32_FIRMWARE target)
  add_custom_command(TARGET ${target}
        POST_BUILD
        COMMAND python3 ${ESPRIT_ROOT}/script/lnCRC32Checksum.py  $<TARGET_FILE:${target}>.bin $<TARGET_FILE:${target}>.ck_bin
        WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
        COMMENT "Generating checksumed file"
    )
ENDMACRO()
include(ln_use_library)

