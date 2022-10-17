#!/bin/bash

VID=$1
PID=$2

sleep 1
for i in `seq 1 10`
do
	lsusb -d $VID:$PID >/dev/null || (echo -ne "." && sleep 1)
done
lsusb -d $VID:$PID || (echo "Device not found" && exit 1)


