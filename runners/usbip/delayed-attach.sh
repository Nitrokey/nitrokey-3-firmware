#!/bin/bash

SCRIPTPATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

function wait_and_attach() {
    echo "PRELAUNCH TASK RUNNING!"
    pushd "$1"

    lsmod | grep vhci-hcd || sudo modprobe vhci-hcd

    endtime=$(($(date +%s) + 10))
    echo "$(date +%s) - $endtime"
    while true; do
        # Check if we've tried long enough and should time out
        if [ $(date +%s) -gt $endtime ]; then
            >&2 echo "Failed to find device before timeout"
            return
        fi
        sleep 0.1
        output=$(sudo usbip list -r "localhost" 2>&1)
        retval=$?
        if [ $retval -eq 0 ]; then
            # The device is available! Now attach it.
            sudo usbip attach -r "localhost" -b "1-1"
            sudo usbip attach -r "localhost" -b "1-1"
            if lsusb | grep -q "Clay Logic Nitrokey 3"; then
                echo "Device attached!"
            else
                >&2 echo "Failed to attach device"
            fi
            return
        elif [ $retval -eq 1 ]; then
            # Couldn't find the port, so keep waiting
            continue
        else
            # Some unexpected error, exit out
            >&2 echo "$output"
            return
        fi
    done
}

# Delete any existing output file
sudo rm -f /tmp/DelayedUSBIPAttach

FUNC=$(declare -f wait_and_attach)

# Run the function as sudo (so it catches sudo login requirement here instead of in the backgroun process).
# Direct all output to the output file so we can review it later if we like.
# Run the command in the background.
sudo bash -c "$FUNC; wait_and_attach \"$SCRIPTPATH\" 2>&1 | tee /tmp/DelayedUSBIPAttach &"
