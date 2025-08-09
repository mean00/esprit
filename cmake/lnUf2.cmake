MACRO(ADD_UF2 tgt)
  find_program ( picotool      NAMES picotool)
  if( "x${picotool}" STREQUAL "xpicotool-NOTFOUND")
    find_program ( elf2uf2      NAMES elf2uf2)
    if( "x${elf2uf2}" STREQUAL "xelf2uf2-NOTFOUND")
      MESSAGE(WARNING "elf2uf2 not found, nor picotool , it is part of the pico SDK. Uf2 will not be generated")
    else()
      add_custom_command(TARGET ${tgt}
            POST_BUILD
            COMMAND elf2uf2  $<TARGET_FILE:${tgt}> $<TARGET_FILE:${tgt}>.uf2
            WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
            COMMENT "Generating uf2 files"
            )
    endif()
  else()
    add_custom_command(TARGET ${tgt}
            POST_BUILD
            COMMAND picotool uf2 convert $<TARGET_FILE:${tgt}> $<TARGET_FILE:${tgt}>.uf2
            WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
            COMMENT "Generating uf2 files"
            )
  endif()
ENDMACRO()

