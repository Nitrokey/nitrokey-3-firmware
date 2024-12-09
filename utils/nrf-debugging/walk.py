# This script walks in a littlefs image and print contents.
#
# E.g.
# % python3 examples/walk.py --img-filename=test.img
# root . dirs ['lfs'] files ['test_directories.py', 'test_files.py', 'test_remove_rename.py', 'test_version.py', 'test_walk.py']
# root ./lfs dirs [] files ['conftest.py', 'test_dir_functions.py', 'test_file_functions.py', 'test_fs_functions.py']
# %

import argparse
import os
import sys

from littlefs import LittleFS

parser = argparse.ArgumentParser()
parser.add_argument("--img-filename", default="lfs.bin")
parser.add_argument("--img-size", type=int, default=81920)
parser.add_argument("--block-size", type=int, default=256)
parser.add_argument("--read-size", type=int, default=4)
parser.add_argument("--prog-size", type=int, default=4)
args = parser.parse_args()

img_filename = args.img_filename
img_size = args.img_size
block_size = args.block_size
read_size = args.read_size
prog_size = args.prog_size

block_count = img_size // block_size
if block_count * block_size != img_size:
    print("image size should be a multiple of block size")
    exit(1)

fs = LittleFS(
    block_size=block_size,
    block_count=block_count,
    read_size=read_size,
    prog_size=prog_size,
)

with open(img_filename, "rb") as f:
    data = f.read()
fs.context.buffer = bytearray(data)

num_files = 1
sum_size = 0

for root, dirs, files in fs.walk("."):
    #print(f"root {root} dirs {dirs} files {files}")
    for dn in dirs:
        info = fs.stat(root + "/" + dn)
        #print(f"{info.size:>10} | {root}/{dn}")
        print(f"{'':>10} | {root}/{dn}")
        num_files += 1
        #sum_size += info.size
    for fn in files:
        info = fs.stat(root + "/" + fn)
        print(f"{info.size:>10} | {root}/{fn}")
        num_files += 1
        sum_size += info.size

blks = 81920 / 256
print()
print (f"size overall: {sum_size} #files: {num_files}  avg_size: {sum_size/num_files}")
print (f"size available: {81920} remaining: {81920-sum_size}")
print (f"blocks: {blks} ... assuming 1 item per block: {blks-num_files}")
