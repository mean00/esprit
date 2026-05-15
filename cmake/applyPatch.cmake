include(FindPatch)

# the patch file is absolute
macro(APPLY_PATCH_IF_NEEDED3 markerFile absPatchFile absSubdir description)
  message(STATUS "Checking for ${absSubdir}/${markerFile}")
  if(NOT EXISTS "${absSubdir}/${markerFile}")
    # MESSAGE(STATUS "   Patching file in ${subdir} ${description}") MESSAGE(STATUS "      patch_file_p(1:::
    # ${absSubdir} <= ${absPatchFile}")
    patch_file_p(1 "${absSubdir}" "${absPatchFile}")
    file(WRITE "${absSubdir}/${markerFile}" "patched")
  else()
    message(STATUS "   Already patched")
  endif()
endmacro()
