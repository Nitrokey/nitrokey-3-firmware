#include <stdio.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/mman.h>
#include <fcntl.h>
#include "lfs.h"

static void *fsmmap = NULL;
static char rndbuf[32] = {
	0,1,2,3,4,5,6,7,
	0,1,2,3,4,5,6,7,
	0,1,2,3,4,5,6,7,
	0,1,2,3,4,5,6,7
};

#include "my_lfs_common.h"
static struct lfs_file LFS_FILE;

int main(int an, char **ac) {
	int r;

	if (an != 2) {
		fprintf(stderr, "Error: output file name missing\n");
		return 1;
	}

	fsmmap = mmap((void *)FILESYSTEM_BASE, FILESYSTEM_SIZE, PROT_READ | PROT_WRITE, MAP_FIXED | MAP_SHARED | MAP_ANONYMOUS, -1, 0);
	if (fsmmap == MAP_FAILED) return 1;

	memset(fsmmap, 0xff, FILESYSTEM_SIZE);

	r = lfs_format(&LFS, &LFS_CONFIG);
	printf("format done, ret: %d\n", r);

	r = lfs_mount(&LFS, &LFS_CONFIG);
	printf("mount done, ret: %d\n", r);

	r = lfs_mkdir(&LFS, "/fido");
	printf("mkdir /fido done, ret: %d\n", r);
	r = lfs_mkdir(&LFS, "/fido/x5c");
	printf("mkdir /fido/x5c done, ret: %d\n", r);
	r = lfs_mkdir(&LFS, "/fido/sec");
	printf("mkdir /fido/sec done, ret: %d\n", r);

	memset(&LFS_FILE, 0, sizeof(LFS_FILE));
	r = lfs_file_open(&LFS, &LFS_FILE, "/fido/x5c/00", LFS_O_WRONLY | LFS_O_CREAT | LFS_O_TRUNC);
	printf("open /fido/x5c/00 done, ret: %d\n", r);
	if (r == 0) {
		char buffer[4096];
		int r2 = open("./fido.crt", O_RDONLY);
		int len = read(r2, buffer, 4096);
		close(r2);

		r = lfs_file_write(&LFS, &LFS_FILE, buffer, len);
		printf("file write done, ret: %d\n", r);
		r = lfs_file_close(&LFS, &LFS_FILE);
		printf("file close done, ret: %d\n", r);
	}

	memset(&LFS_FILE, 0, sizeof(LFS_FILE));
	r = lfs_file_open(&LFS, &LFS_FILE, "/fido/sec/00", LFS_O_WRONLY | LFS_O_CREAT | LFS_O_TRUNC);
	printf("open /fido/sec/00 done, ret: %d\n", r);
	if (r == 0) {
		char buffer[4096];
		int r2 = open("./fido.key", O_RDONLY);
		int len = read(r2, buffer, 4096);
		close(r2);

		r = lfs_file_write(&LFS, &LFS_FILE, buffer, len);
		printf("file write done, ret: %d\n", r);
		r = lfs_file_close(&LFS, &LFS_FILE);
		printf("file close done, ret: %d\n", r);
	}

	r = lfs_mkdir(&LFS, "/trussed");
	printf("mkdir /trussed done, ret: %d\n", r);
	r = lfs_mkdir(&LFS, "/trussed/dat");
	printf("mkdir /trussed/dat done, ret: %d\n", r);
	memset(&LFS_FILE, 0, sizeof(LFS_FILE));
	r = lfs_file_open(&LFS, &LFS_FILE, "/trussed/dat/rng-state.bin", LFS_O_WRONLY | LFS_O_CREAT | LFS_O_TRUNC);
	printf("open /trussed/dat/rng-state.bin done, ret: %d\n", r);
	if (r == 0) {
		r = lfs_file_write(&LFS, &LFS_FILE, rndbuf, 32);
		printf("file write done, ret: %d\n", r);
		r = lfs_file_close(&LFS, &LFS_FILE);
		printf("file close done, ret: %d\n", r);
	}

	r = lfs_unmount(&LFS);
	printf("unmount done, ret: %d\n", r);

	r = open(ac[1], O_WRONLY | O_CREAT | O_EXCL, 0644);
	write(r, fsmmap, FILESYSTEM_SIZE);
	close(r);
	printf("file written to fd %d\n", r);

	return 0;
}
