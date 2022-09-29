
# Multi-Factor Authentication Device (USB key) Firmware

This repository the firmware of  Multi-Factor Authentication Device (USB key).

Please use the proto2-dev2 branch for current measurements and further development of the Multi-Factor Authentication Device.

For Multi-Factor Authentication Device (USB key) Firmware please work in following folder:

nitrokey-3-firmware-proto2-power-test/runners/embedded

#############
## About

The  Multi-Factor Authentication Device (USB key) firmware  is written in Rust.

It uses  the Trussed firmware framework.


#############
## Documentation

Documentation for Multi-Factor Authentication Device (USB key) is not yet complete made.

Please see The Installation Guide and User Guide down below.

Documentation for Nitrokey 3 firmware users is available in the Nitrokey 3 section on docs.nitrokey.com. 

For developer documentation for Nitrokey 3 firmware see the docs directory.

#############
## License

This software is fully open source.

All software, unless otherwise noted, is dual licensed under Apache 2.0 and MIT. 

You may use the software under the terms of either the Apache 2.0 license or MIT license.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.


#############


## Requirements

    • Multi-Factor Authentication Device (USB key)  Prototype 1 or  
    • Multi-Factor Authentication Device (USB key) Prototype  2 or
    • nrfDKDeveleopment Board  

    • Rust,
    • Python 
    • and othere Software Dependecies (see the List down below).
    
#############
## Basic Commands for building and running

Clone the firmware repository from Gitlab/ Github to your desktop.


Work in following folder: 

nitrokey-3-firmware-proto2-power-test/runners/embedded 

$ cd nitrokey-3-firmware-proto2-power-test/runners/embedded

#
Prototype 1:

Building:

$ make build-proto1 

Running:

$ make reset-proto1 running 

#
Prototype 2:

Building:

$ make build-proto2

Running:

$ make reset-proto2

#
Nordic nrfdk Development Board

Building:

$ make build-nrfdk


Running:

$ make reset-nrfdk


#
Nk3am Board 

Building:

$ make build-nk3am

Running:

$ make reset-nk3am

#
Start rtt viewer and logger

Open a Terminal and type
 
$ JLinkRTTViewerExe

Follow the Instructions

connect directly  via usb or  put the serial number of the debugger to connect.

#
Display the serial numbers of all the debuggers connected to the PC.

$ nrfjprog -i



#############
## Dependencies of the Multi-Factor Authentication Device (USB key) 

To build the firmware from source, you need these dependencies for the Debian/Ubuntu/Linux Systems:


    • rust and cargo
    • cargo-binutilis
    • lvm-tools-preview
    • flip-link
    • Clang with development headers
    • GCC Compiler 
    • libudev-dev
    • FFMPEG

    • python 3.8.
    • pip for python3.8
    • Pillow (for Python)
    • TOML (for Python)

    • J-Link Software /Segger Tools
    • nRF Command Line Tools

    • rust-std component for  the target


Optional if you are using VSCODE:

    • VSCODE
    • Rust Analyzer Plug-In
    • Better TOML Plug In
    • Python Extension Pack
    

Optional if you want to debugg with OpenOcd or gdb

    • gdb-multiarch
    • minicom  
    • OpenOCD  
    • gdb
    • gdb-arm-none-eabi 
    • arm-none-eabi-gdb
    • itmdump  
    • cargo-embed

If your computer has Bluetooth functionality  you can additionally install these tools to try out  the Bluetooth functions of the prototypes. 

All these are optional and if you don't already have a Bluetooth manager application like Blueman. 

    • bluez 
    • rfkill 

Optional for   Smartphone to test the  Bluetooth functionality

    • nRF Connect for Mobile.



#############
## Installation Instructions for Debian/Ubuntu/Linux  based Systems:

####

Update and Upgrade your  Debian/Ubuntu/Linux  System:

$ sudo apt-get update && sudo apt-get upgrade


#############

## Install Rust, Cargo and Rust tools 

Base Rust installation

Go to 

https://rustup.rs

#
Run the following in your terminal:

$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

and follow the instructions.

#
Verify Installation:

$ rustc --version

#
Update: 

$ rustup update

#
Verify Cargo Installation:

$ cargo –version




#
Creating a Project with Cargo

$ cargo new project_name

#
Building  a Cargo Project

$ cargo build 

#
Running a Cargo Project

$ cargo run

#
Checking a Cargo Project 

$ cargo check

#
Further Information:

https://doc.rust-lang.org/book/ch01-01-installation.html
 
#
To automatically update the software installed through cargo install, you can use the cargo-update crate:

$ cargo install cargo-update

$ cargo install-update -a

#############

## Install cargo-binutils

$ cargo install cargo-binutils


#############

## Install llvm-tools-preview


$ rustup component add llvm-tools-preview

#
Further Information:

https://github.com/rust-embedded/cargo-binutils


#############

## Install flip-link

$ cargo install flip-link

#
Further Information:
#
flip-link is available on crates.io. 

https://github.com/knurling-rs/flip-link


#############

## Install clang with development headers

 
Run the following in your terminal:

$ sudo apt-get install llvm clang libclang-dev gcc-arm-none-eabi libc6-dev-i386


#############

## Install GCC Compiler 
#
Start by updating the packages list:

$ sudo apt update

#
Install the build-essential package.

The command installs a bunch of new packages including gcc, g++ and make.

$ sudo apt install build-essential

#
You may also want to install the manual pages about using GNU/Linux for development:

$ sudo apt-get install manpages-dev

#
To validate that the GCC compiler is successfully installed, use the gcc --version command which prints the GCC version:

$ gcc --version

#############
## Install libudev-dev

$ sudo apt-get install libudev-dev libusb-1.0-0-dev


#############

## Install Ffmpeg 4
#
Step 1: System update

$ sudo apt update

#
Step 2: Install FFmpeg

$ sudo apt install ffmpeg

#
Step 3: Verify installation

$ ffmpeg -version

#
Step 4: Print  all avilable FFmpeg’s encoders and decoders :

$ ffmpeg -encoders

 
#############

## Install python 3.8.
#
Verify if  python is installed


Run the following in your terminal:

$ python --version

#
If not installed run:

Step 1: Update and Refresh Repository Lists

$ sudo apt update

#
Step 2: Install Supporting Software

The software-properties-common package gives you better control over your package manager by letting you add PPA (Personal Package Archive) repositories. 

Install the supporting software with the command:

$ sudo apt install software-properties-common

#
Step 3: Add Deadsnakes PPA

Deadsnakes is a PPA with newer releases than the default Ubuntu repositories. 

Add the PPA by entering the following:

$ sudo add-apt-repository ppa:deadsnakes/ppa

#
Refresh the package lists again:

$ sudo apt update

#
Step 4: Install Python 3

Now you can start the installation of Python 3.8 with the command:

$ sudo apt install python3.8

#
Allow the process to complete and verify the Python version was installed sucessfully::

$ python --version


################

## Install pip
#
Use this command to see if pip is installed : 

$ python -m pip –version

#
If pip is not installed you can install it with : 

$  sudo apt install python-pip

#
or for Python 3 :

$  sudo apt install python3-pip


###############

## Install Pillow (for Python)
#
$ python3 -m pip install --upgrade pip

$ python3 -m pip install --upgrade Pillow


###############


## Install TOML (for Python) 

#
To install the latest release on PyPI, simply run:

$ pip install toml


#############

## Install SEGGER Software /J-Link Software 
#
Choose your installer and follow the Instructions.

https://www.segger.com/downloads/jlink/


#############

## Installing the nRF Command Line Tools
#
Choose your installer and follow the Instructions.


https://infocenter.nordicsemi.com/index.jsp?topic=%2Fug_nrf_cltools%2FUG%2Fcltools%2Fnrf_nrfjprogexe_reference.html


https://www.nordicsemi.com/Products/Development-tools/nrf-command-line-tools/download#infotabs


#############

## Install the rust-std component for the target
#
Following command  will show all the supported compilation targets.

$ rustup target list

#
Identify the microcontroller, its processor architecture and sub-architecture.

This information should be in the device's data sheet or manual. 
In the case of the nRF52840, the processor is an ARM Cortex-M4 core. 
Select a compatible compilation  target from the step above. 


The ARM Cortex-M ISA is backwards compatible so for example you could compile a program using the thumbv6m-none-eabi and run it on an ARM Cortex-M4 microcontroller. 

 $ rustup target add thumbv7em-none-eabi 

#
This will work but using the thumbv7em-none-eabihf results in better performance (ARMv7-M instructions will be emitted by the compiler) so it should be preferred.

$ rustup target add thumbv7em-none-eabihf


#############
 
Optional:

## Install VSCODE:
#
Installation 
See the Download Visual Studio Code page for a complete list of available installation options.

The easiest way to install Visual Studio Code for Debian/Ubuntu based distributions is to download and install the .deb package (64-bit), either through the graphical software center if it's available, or through the command line with:

$ sudo apt install ./<file>.deb


https://code.visualstudio.com/download

https://code.visualstudio.com/docs/setup/linux


#############

Optional:

## Install Rust Analyzer

If you are using Visual Studio Code, we recommend you install Rust Analyzer to help you during development.

Launch VS Code install   Rust-Analyzer in Extensions.
#
Or  Launch VS Code Quick Open (Ctrl+P), paste the following command, and press enter.

$ ext install rust-lang.rust-analyzer
#
Further Information:

https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer

#
If you are using Rust-Anaylzer with VSCODE you have to configure the settings.json file :
#
$ sudo nano embedded/.vscode/settings.json
#

or open it directly in VSCODE
#
{
"makefile.extensionOutputFolder": "./.vscode",

"rust-analyzer.checkOnSave.allTargets": false,

"rust-analyzer.cargo.noDefaultFeatures": true,

"rust-analyzer.cargo.features": ["board-proto2", "develop"],

"rust-analyzer.cargo.target": "thumbv7em-none-eabihf"

} 


##################

 Optional 

## Install gdb-multiarch,minicom, openocd, gdb,  gdb-arm-none-eabi ,  arm-none-eabi-gdb,itmdump,cargo-embed ,bluez,rfkill, mboot

$ sudo apt-get install gdb-multiarch
#
$ sudo apt-get install minicom
#
$ sudo apt-get install openocd
#
$ sudo apt-get install gdb 
#
$ sudo apt-get install gdb-arm-none-eabi
#
$ sudo apt-get install arm-none-eabi-gdb


Create /etc/udev/rules.d/99-openocd.rules for nrf and reload them

#
Install itmdump

$ cargo install itm

#
Verify the version 

$  itmdump -V

#
Install cargo-embed:

$ cargo install cargo-embed --vers 0.11.0

#
Install bluez, rfkill
#
$ sudo apt-get install bluez 
#
$ sudo apt-get install rfkill

#
Install mboot

$ pip install mboot


Further Information:

https://github.com/molejar/pyMBoot


Optionally, add and activate the mboot udev roles so that you don’t need root privileges for flashing:
#
$ curl https://raw.githubusercontent.com/molejar/pyIMX/master/udev/90-imx-sdp.rules > /etc/udev/rules.d/90-imx-sdp.rules
#
$ sudo udevadm control --reload-rules
#
#
#
#



####################################################################################################################################################################


# Additional Information to the original Nitrokey 3 Firmware
This repository contains the firmware of Nitrokey 3 USB keys.

## About

The Nitrokey 3 firmware is written in [Rust][].  It uses the [Trussed][] firmware framework and is developed in collaboration with [SoloKeys][] (see the [solo2][] repository).

[Rust]: https://rust-lang.org
[Trussed]: https://trussed.dev/
[SoloKeys]: https://solokeys.com/
[solo2]: https://github.com/solokeys/solo2

## Documentation

Documentation for users is available in the [Nitrokey 3 section on docs.nitrokey.com][docs.nitrokey.com].

[docs.nitrokey.com]: https://docs.nitrokey.com/nitrokey3/index.html

This documentation is available for developers and testers:
- [Quickstart Guide](./docs/quickstart.md): Compiling and flashing the firmware
- [Troubleshooting Guide](./docs/troubleshooting.md): Solving common development issues
- [Contributing Guide](./docs/contributing.md): Contributing to this repository
- [Maintenance Guide](./docs/maintenance.md): Maintaining this repository
- [Testing Guide](./docs/testing.md): Testing beta firmware versions

## Dependencies

To build the firmware from source, you need these dependencies:

- Rust (current stable release for the `thumbv8m.main-none-eabi` target with the `llvm-tools-preview` component)
- clang with development headers
- [`flip-link`][]
- [`cargo-binutils`][]

[`flip-link`]: https://github.com/knurling-rs/flip-link
[`cargo-binutils`]: https://github.com/rust-embedded/cargo-binutils

To flash the firmware to the device, you need [`mboot`][] or [`lpc55`][].

[`mboot`]: https://github.com/molejar/pyMBoot
[`lpc55`]: https://github.com/lpc55/lpc55-host

## License

This software is fully open source.

All software, unless otherwise noted, is dual licensed under [Apache 2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT).
You may use the software under the terms of either the Apache 2.0 license or MIT license.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
