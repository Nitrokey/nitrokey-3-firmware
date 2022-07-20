#if !defined(FILESYSTEM_BASE) || !defined(FILESYSTEM_SIZE) || !defined(FILESYSTEM_BLK_SHIFT)
# error Definitions for FILESYSTEM_BASE and FILESYSTEM_SIZE and FILESYSTEM_BLK_SHIFT missing.
#endif

#define	FS_BLKSIZE (1U << FILESYSTEM_BLK_SHIFT)

static int mm_read(const struct lfs_config *c, lfs_block_t b, lfs_off_t o, void *buf, lfs_size_t sz) {
        printf("F RD %02x+%04x %04x (%p)\n", b, o, sz, __builtin_return_address(1));
        (void)c;
        memcpy(buf, ((char *)fsmmap) + (b << FILESYSTEM_BLK_SHIFT) + o, sz);
        return LFS_ERR_OK;
}

static int mm_prog(const struct lfs_config *c, lfs_block_t b, lfs_off_t o, const void *buf, lfs_size_t sz) {
        printf("F WR %02x+%04x %04x (%p)\n", b, o, sz, __builtin_return_address(1));
        (void)c;
        memcpy(((char *)fsmmap) + (b << FILESYSTEM_BLK_SHIFT) + o, buf, sz);
        return LFS_ERR_OK;
}

static int mm_erase(const struct lfs_config *c, lfs_block_t b) {
        printf("F ER %02x\n", b);
        memset(((char *)fsmmap) + (b << FILESYSTEM_BLK_SHIFT), 0xff, c->block_size);
        return LFS_ERR_OK;
}

static int mm_noop(const struct lfs_config *c) {
        (void)c;
        return LFS_ERR_OK;
}

static struct lfs LFS;
static struct lfs_config LFS_CONFIG = {
        .context = NULL,
        .read = mm_read,
        .prog = mm_prog,
        .erase = mm_erase,
        .sync = mm_noop,
#ifdef LFS_THREADSAFE
        .lock = mm_noop,
        .unlock = mm_noop,
#endif
        .read_size = 4,
        .prog_size = 4,
        .block_size = FS_BLKSIZE,
        .block_count = FILESYSTEM_SIZE >> FILESYSTEM_BLK_SHIFT,
        .block_cycles = -1,
        .cache_size = 256,
        .lookahead_size = 8,
        .read_buffer = NULL,
        .prog_buffer = NULL,
        .lookahead_buffer = NULL,
        .name_max = 0,
        .file_max = 0,
        .attr_max = 0,
        .metadata_max = 0
};

