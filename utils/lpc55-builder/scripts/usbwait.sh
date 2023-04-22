#!/bin/bash

echo -n "Waiting for USB device: $@ "
sleep 1
for i in `seq 1 10`
do
	for id in $@
	do
		if lsusb -d $id
		then
		  sleep 2
			exit 0
		fi
		echo -n "."
		sleep 1
	done
done

echo
echo "Error: Device not found" >&2
exit 1
