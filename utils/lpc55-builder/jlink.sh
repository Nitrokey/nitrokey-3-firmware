#!/bin/sh

if [ \( $# -gt 1 \) -o \( "$1" = "-h" \) -o \( "$1" = "--help" \) -o \( "$1" = "help" \) ]
then
	echo "Usage: $0 [command_file]" >&2
	exit 1
fi

args="-device LPC55S69_M33_0 -if SWD -autoconnect 1 -speed 4000 -NoGui 1 -ExitOnError 1"
if [ $# -eq 1 ]
then
	args="$args -CommandFile $1"
fi

exec JLinkExe $args
