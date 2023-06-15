
binary:
	@echo "please symlink the compiled binary"
	@echo "likely somewhere here: ../../runners/embedded/artifacts/"
	@echo "maybe one of these:"
	ls -la ../../runners/embedded/artifacts/*.bin

symbols.txt:
	@echo "please symlink the compiled symbols.txt file"
	@echo "likely somewhere here: ../../runners/embedded/"
	@echo "maybe one of these:"
	ls -la ../../runners/embedded/sym*.txt

ocd: ocd.conf
	openocd -f ocd.conf

bt: gdb_bt.cmds binary
	arm-none-eabi-gdb --batch -x gdb_bt.cmds

regs: gdb_regs.cmds binary
	arm-none-eabi-gdb --batch -x gdb_regs.cmds

it: gdb.cmds
	arm-none-eabi-gdb -x gdb.cmds

rtt:
	python ocdtool.py r
	nc localhost 4999

framedump: symbols.txt
	python ocdtool.py f

lfs_show:
	nrfjprog --memrd 0xd8000 --n 81920

lfs_dump:
	nrfjprog --readcode temp-memory.bin
	dd if=temp-memory.bin of=lfs.bin bs=1 skip=884736 count=81920
	python walk.py

lfs_fast:
	nrfjprog --memrd 0xd8000 --n 81920 > test.bin
	python parse-memrd.py test.bin
	ls -la output.bin
	python walk.py --img-filename output.bin
