#include <stdio.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/mman.h>
#include <fcntl.h>
#include "lfs.h"

static void *fsmmap = NULL;
#include "my_lfs_common.h"

static void recurse(int depth, char *dn) {
	int r;
	struct lfs_dir ldir;
	struct lfs_info linfo;

	memset(&ldir, 0, sizeof(ldir));

	r = lfs_dir_open(&LFS, &ldir, dn);
	if (r != LFS_ERR_OK) { printf("dirO %s %d\n", dn, r); exit(1); }

	while (1) {
		memset(&linfo, 0, sizeof(linfo));
		r = lfs_dir_read(&LFS, &ldir, &linfo);
		if (r == 0) break;
		if (r < 0) { printf("dirR %s %d\n", dn, r); exit(1); }

		if ((linfo.type == LFS_TYPE_DIR) && (!strcmp(linfo.name, ".") || !strcmp(linfo.name, ".."))) {
			continue;
		}

		printf("%.*s+ %c %06x %s\n", depth*2, "         ", linfo.type == LFS_TYPE_REG ? 'f' : 'd', linfo.type == LFS_TYPE_REG ? linfo.size : 0, linfo.name);

		if (linfo.type == LFS_TYPE_DIR) {
			char next_dn[LFS_NAME_MAX+1];
			snprintf(next_dn, LFS_NAME_MAX, "%s/%s", dn, linfo.name);
			recurse(depth+1, next_dn);
		}
	}

	r = lfs_dir_close(&LFS, &ldir);
	if (r != LFS_ERR_OK) { printf("dirC %s %d\n", dn, r); exit(1); }
}
	
int main(int an, char **ac) {
	int r;

	int f = open(ac[1], O_RDONLY);
	fsmmap = mmap((void *)FILESYSTEM_BASE, FILESYSTEM_SIZE, PROT_READ, MAP_FIXED | MAP_SHARED, f, 0);
	if (fsmmap == MAP_FAILED) return 1;

	r = lfs_mount(&LFS, &LFS_CONFIG);
	if (r != LFS_ERR_OK) { printf("mount %d\n", r); exit(1); }

	recurse(0, "/");

	r = lfs_unmount(&LFS);
	if (r != LFS_ERR_OK) { printf("unmount %d\n", r); exit(1); }

	return 0;
}
