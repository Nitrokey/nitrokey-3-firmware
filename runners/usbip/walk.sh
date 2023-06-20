#!/bin/sh

echo "INTERNAL"
python ../../utils/debugging/walk.py --img-filename fido_ifs --block-size 512 --read-size=16 

echo "EXTERNAL"
python ../../utils/debugging/walk.py --img-filename fido_efs --block-size 4096 --read-size=4 
