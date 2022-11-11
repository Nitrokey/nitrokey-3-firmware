#!/bin/sh

devices=`lsusb -d 1fc9:0021 ; lsusb -d 20a0:42dd ; lsusb -d 20a0:42b2`
echo "$devices"
num_devices=`echo "$devices" | wc -l`

if [ $num_devices -eq 0 ]
then
	echo "Error: No lpc55 Nitrokey 3 device connected" >&2
	exit 1
fi

if [ $num_devices -gt 1 ]
then
	echo "Error: Multiple Nitrokey 3 device connected" >&2
	exit 1
fi

if lsusb -d 20a0:42b2 > /dev/null
then
	nitropy nk3 reboot --bootloader
fi
