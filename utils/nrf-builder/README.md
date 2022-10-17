### nRF52 / NK3AM Building Utils

* just build all artifacts: `make` or `make build`

* flash specific parts:
  * `make flash-provisioner`
	* `make flash-firmware`
	* `make flash-bootloader`
	
* provision test-keys/certs: `make provision-keys`

* exercise a full round:
  * build it all
  * flash bootloader
	* flash provisioner
	* provision keys
	* flash firmware
	* just: `make full-deploy`
 
* clean up the mess: `make clean` 
