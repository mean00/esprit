include(ln_merge_libs)

macro(GENERATE_GD32_FIRMWARE target)
  ln_merge_libs()
  configure_file("${LN_MCU_FOLDER}/boards/${LN_BOARD_NAME}/rp2350_linker.ld.in" "${CMAKE_BINARY_DIR}/linker_script.ld"
                 @ONLY)
  add_executable(${target} ${ARGN})
  target_sources(${target} PRIVATE ${LN_MCU_FOLDER}/conf/bs2_default_padded_checksummed.S)
  target_link_libraries(${target} PUBLIC esprit_dev esprit_single_lib) # duplicates are NOT a mistake !
  target_link_libraries(${target} PUBLIC pico_stdlib)
  target_link_directories(${target} PUBLIC ${CMAKE_BINARY_DIR})
  if(LN_CUSTOM_LD_SCRIPT)
    set(SCRIPT
        ${LN_CUSTOM_LD_SCRIPT}
        CACHE INTERNAL "")
  else()
    set(SCRIPT
        "${CMAKE_BINARY_DIR}/linker_script.ld"
        CACHE INTERNAL "")
  endif()

  add_custom_command(
    TARGET ${target}
    POST_BUILD
    COMMAND ${CMAKE_OBJCOPY} -Obinary $<TARGET_FILE:${target}> $<TARGET_FILE:${target}>.bin
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Generating bin file")
  include(lnUf2)
  add_uf2(${target})
endmacro()

include(ln_use_library)

macro(HASH_GD32_FIRMWARE target)

endmacro()
