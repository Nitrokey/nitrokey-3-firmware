#!/usr/bin/python3

import os
import struct
import subprocess
import sys

def get_img_dim(fn):
	p = subprocess.Popen("identify %s " % fn, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, shell=True, text=True)
	pout, perr = p.communicate()
	assert perr is None
	assert p.returncode == 0
	dim = pout.strip().split(" ")[2].split("x")
	# dimension may be "144x216" or "144x216+0+0" - cut off "+0+0" if present
	if "+" in dim[1]:
		dim[1] = dim[1].split("+")[0]
	return int(dim[0]), int(dim[1])

SEEN_RGBVALS = {}
"""Convert an image from PNG to raw RGB565 pixel data, optionally reordering
   the pixels so that the contained sprites (each cw*ch pixels wide, tiled
   on a cntx*cnty grid) are placed into a single 1*ch pixels wide column.
"""
def png_to_raw565(fn, cw=None, ch=None, cntx=None, cnty=None):
	iw, ih = get_img_dim(fn)
	if cw is None:
		cw = iw
		ch = ih
		cntx = 1
		cnty = 1
  
	else:
		assert iw == cw*cntx
		assert ih == ch*cnty
	if iw % 2 != 0:
		iwpadded = iw + 1
	else:
		iwpadded = iw
	fnbmp = fn.replace(".png", ".bmp")
	fnraw = fn.replace(".png", ".raw")

	# some imagemagick versions ignore 565 flag, so use ffmpeg instead
	p = subprocess.Popen("ffmpeg -i %s -pix_fmt rgb565 %s" % (fn, fnbmp),
			stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
			shell=True)
	p.communicate()
	assert p.returncode == 0

	fo = open(fnraw, "wb")
	with open(fnbmp, "rb") as fb:
		fb.seek(0x0a)
		pxoff, = struct.unpack("<I", fb.read(4))
		fb.seek(0x12)
		iw_, ih_ = struct.unpack("<II", fb.read(8))
		assert iw_ == iw and ih_ == ih
		fb.seek(0x22)
		pxsz, = struct.unpack("<I", fb.read(4))
		assert pxsz == iwpadded*ih*2
		fb.seek(pxoff)
		bmpdata = fb.read(pxsz)
	assert len(bmpdata) == iwpadded*ih*2

	for i in range(cntx*cnty):
		charposx = i % cntx
		charposy = i // cntx
		bytepos = (ih-1-(charposy*ch))*iwpadded*2 + (charposx*cw)*2
		for y in range(ch):
			rowpos = bytepos - y*2*iwpadded
			for x in range(cw):
				rgbval, = struct.unpack("<H", bmpdata[rowpos+2*x:rowpos+2*x+2])
				rval = (rgbval >> 11) & 0x1f
				gval = (rgbval >> 5) & 0x3f
				bval = (rgbval & 0x1f)
				if rgbval not in SEEN_RGBVALS.keys():
					print("New RGB value: %04x => (%02x, %02x, %02x)" % (rgbval, rval, gval, bval))
					SEEN_RGBVALS[rgbval] = True
				reverseval = (bval << 11) | (gval << 5) | rval
				# write with reversed components
				## fo.write(struct.pack("<H", reverseval))
				# write big-endian, original order
				fo.write(struct.pack(">H", rgbval))
			# fo.write(bmpdata[rowpos:rowpos+2*cw])

	fo.close()
	# comment this out to debug PNG->BMP->RAW conversion
	os.unlink(fnbmp)

png_to_raw565("font_9x18.png", 9, 18, 16, 12)
png_to_raw565("font_9x18_bold.png", 9, 18, 16, 12)
png_to_raw565("texmap.png")
sys.exit(0)
