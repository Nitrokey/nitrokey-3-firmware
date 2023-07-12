#!/bin/sh

CFG_PATH="$1"
BUILD_PROFILE="$2"
BOARD="$3"


TMP_CFG=$(mktemp)
cp "$CFG_PATH" $TMP_CFG
echo '' >> $TMP_CFG
echo '[build]' >> $TMP_CFG
echo "build_profile = \"$BUILD_PROFILE\"" >> $TMP_CFG
echo "board = \"$BOARD\"" >> $TMP_CFG

diff $TMP_CFG cfg.toml 
if [ "$?" != 0 ]; then
  mv -f $TMP_CFG cfg.toml
else
  rm $TMP_CFG
fi
