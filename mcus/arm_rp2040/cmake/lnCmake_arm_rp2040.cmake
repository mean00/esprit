include(ln_merge_libs)

MACRO(GENERATE_GD32_FIRMWARE target)
  configure_file( "${LN_MCU_FOLDER}/boards/${LN_BOARD_NAME}/rp2040_linker.ld.in" "${CMAKE_BINARY_DIR}/linker_script.ld" @ONLY)
  LN_MERGE_LIBS()
  ADD_EXECUTABLE(${target} ${ARGN} )
  TARGET_SOURCES(${target} PRIVATE ${LN_MCU_FOLDER}/conf/bs2_default_padded_checksummed.S)
  TARGET_LINK_LIBRARIES(${target} PUBLIC pico_stdlib)
  TARGET_LINK_LIBRARIES(${target} PUBLIC esprit_single_lib esprit_dev)
  target_link_directories(${target} PUBLIC ${CMAKE_BINARY_DIR})
  IF(LN_CUSTOM_LD_SCRIPT)
    SET(SCRIPT ${LN_CUSTOM_LD_SCRIPT} CACHE INTERNAL "")
  ELSE()
    SET(SCRIPT "${CMAKE_BINARY_DIR}/linker_script.ld" CACHE INTERNAL "")
  ENDIF()

  add_custom_command(TARGET ${target}
                   POST_BUILD
                   COMMAND ${CMAKE_OBJCOPY} -Obinary $<TARGET_FILE:${target}> $<TARGET_FILE:${target}>.bin
                   WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
                   COMMENT "Generating bin file"
    )
  include(lnUf2)
  ADD_UF2(${target})
ENDMACRO(GENERATE_GD32_FIRMWARE target)

include(ln_use_library)

MACRO(HASH_GD32_FIRMWARE target)
ENDMACRO()
