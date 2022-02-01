LITTLEFS_DIR := /code/_upstream/littlefs
LITTLEFS_BASE := 0xe0000

fs.hex:	fs.raw
	objcopy -I binary -O ihex --adjust-vma $$(( $(LITTLEFS_BASE) )) $^ $@
	@wc -l $@
	sed -i -e '/^:10....00FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF../ d' -e '/^:......03/ d' $@
	@wc -l $@

fs.raw: gen_init_fs
	rm -f $@
	./$^ $@

gen_init_fs: gen_init_fs.c
	gcc -g2 $^ -I$(LITTLEFS_DIR) $(LITTLEFS_DIR)/lfs.a -o $@

list_fs: list_fs.c
	gcc -g2 $^ -I$(LITTLEFS_DIR) $(LITTLEFS_DIR)/lfs.a -o $@
