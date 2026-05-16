#!/bin/bash
#set -x
#
if [ "$#" -lt 3 ]; then
  echo "Usage: rustgen.sh xxx.h xxx.rs [extra_dir] [extra_dir2] [blocklist]"
  exit 1
fi
if [ "$#" -gt 6 ]; then
  echo "Usage: rustgen.sh xxx.h xxx.rs [extra_dir] [extra_dir2] [blocklist]"
  exit 1
fi
ME=$0
export ME=$(realpath $ME)
export ME=$(dirname $ME)
bash $ME/rustgen_lang.sh $1 $2 c++ $3 $4 $5 $6
