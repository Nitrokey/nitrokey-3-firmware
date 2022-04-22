#!/usr/bin/python3

import os
import PIL
import PIL.Image
import PIL.ImageColor
import sys

def rgb565(rgb):
	r, g, b = rgb
	r >>= 3
	g >>= 2
	b >>= 3
	return (r << 11) | (g << 5) | b

class Palette(object):
	SPRITE_PALETTE_DEF = """const %s: SpritePalette = {
	let colors: &[u16; %d] = &[%s];
	SpritePalette { bpp: %d, colors }
};"""

	def __init__(self, name, colors):
		self.name = name
		# primary palette information
		self.colors = []
		# RGB->RGB565 conversion table
		self.list = {}
		# LUT: cache for nearest-neighbor RGB->RGB lookup
		self.lut = {}
		for c in colors:
			cv = PIL.ImageColor.getrgb(c)
			self.colors.append(cv)
			self.list[cv] = rgb565(cv)
		# calculate required BPP
		l = 2*len(self.colors)-1
		i = 0
		while l > 1:
			i += 1
			l >>= 1
		self.bpp = i
		self.emitted = False

	def emit(self):
		swab = lambda v: ((v & 0xff00) >> 8) | ((v & 0xff) << 8)
		colstr = ", ".join(["0x%04x" % swab(self.list[c]) for c in self.colors])
		print(self.SPRITE_PALETTE_DEF % (self.name, len(self.colors), colstr, self.bpp))
		self.emitted = True

	def closest(self, rgb, max_dist):
		if rgb in self.lut:
			return self.lut[rgb]
		closest = (max_dist, None)
		for k in self.colors:
			d = (k[0] - rgb[0])**2 + (k[1] - rgb[1])**2 + (k[2] - rgb[2])**2
			if d <= closest[0]:
				closest = (d, k)
		self.lut[rgb] = closest[1]
		assert closest[1] is not None, "color %s not in palette, delta %d" % (str(rgb), closest[0])
		return closest[1]

def read_palette(pilimg):
	print(pilimg.getcolors())
	assert False, "Dynamic Palette Discovery not yet supported"

def _mash_pixels(pixelgroup, bpp):
	v = 0
	if bpp == 1:
		for i in range(16):
			v |= (pixelgroup[i] << i)
	elif bpp == 2:
		for i in range(8):
			v |= (pixelgroup[i] << (2*i))
	elif bpp == 3:
		for i in range(5):
			v |= (pixelgroup[i] << (3*i))
	elif bpp == 4:
		for i in range(4):
			v |= (pixelgroup[i] << (4*i))
	else:
		assert False, "Pixel Mashing not supported for this BPP"
	assert v <= 0xffff, "MASH? %s --> %08x" % (pixelgroup, v)
	return v

def mash_pixels(pixels, bpp):
	pixels = list(pixels)
	data = []
	ppw = 16//bpp
	while len(pixels) % ppw != 0:
		pixels.append(0)
	for i in range(0, len(pixels), ppw):
		data.append(_mash_pixels(pixels[i:i+ppw], bpp))
	return data

def pixel_histogram(pixels):
	histo = {}
	for p in pixels:
		if p in histo:
			histo[p] += 1
		else:
			histo[p] = 1
	return histo

class Image(object):
	SPRITE_MAP_DEF = """pub const %s: SpriteMap = {
	let sprites: &[&Sprite; %d] = &[%s];
	SpriteMap {
		width: %d,
		height: %d,
		count_x: %d,
		count_y: %d,
		palette: &%s,
		sprites
	}
};"""
	SPRITE_DEF = """const %s_%d_%d: Sprite = Sprite {
	data: &%s
};"""

	def __init__(self, name, fn, palette, sprite_dim=None):
		self.name = name
		self.source = PIL.Image.open(fn)
		self.source_bands = self.source.getbands()
		self.w = self.source.width
		self.h = self.source.height
		self.palette = palette or read_palette(self.source)
		if sprite_dim is None:
			if self.w > self.h:
				self.sprite_dim = (min(64, self.w), min(16, self.h))
			else:
				self.sprite_dim = (min(16, self.w), min(64, self.h))
		else:
			assert sprite_dim[0] * sprite_dim[1] <= 1024
			self.sprite_dim = sprite_dim
		self.cnt_x = self.w // self.sprite_dim[0]
		self.cnt_y = self.h // self.sprite_dim[1]
		assert self.w % self.sprite_dim[0] == 0
		assert self.h % self.sprite_dim[1] == 0
		color_count = len(self.source.getcolors(4096) or [])
		if color_count > 2**self.palette.bpp:
			print("// %s: source has %d > 2^%d colors, approximating" % (name, len(self.source.getcolors(4096)), self.palette.bpp))
			self.color_approx = 0x3ffffff
		else:
			self.color_approx = 0

	def emit(self):
		if not self.palette.emitted:
			self.palette.emit()

		sprite_deflist = []
		for y in range(self.cnt_y):
			y0 = y*self.sprite_dim[1]
			y1 = (y+1)*self.sprite_dim[1]
			for x in range(self.cnt_x):
				x0 = x*self.sprite_dim[0]
				x1 = (x+1)*self.sprite_dim[0]
				subsource = self.source.crop((x0, y0, x1, y1))
				pixels = list(subsource.getdata())
				# pixels[] is a list of tuples according to source image band information
				if self.source_bands == ('L',):
					pixels = map(lambda p: self.palette.closest((p, p, p), self.color_approx), pixels)
				elif self.source_bands == ('R','G','B','A') or self.source_bands == ('R','G','B'):
					pixels = map(lambda p: self.palette.closest((p[0], p[1], p[2]), self.color_approx), pixels)
				else:
					assert False, "Unsupported Source Data"
				# pixels[] is a list of RGB values approximated to indicated palette
				pixels = map(lambda p: self.palette.colors.index(p), pixels)
				# pixels[] is a list of palette indices
				pixels = list(pixels)
				if self.color_approx > 0:
					histo = pixel_histogram(pixels)
					print("// Histogram: %s" % str(histo))
				pixeldata = mash_pixels(pixels, self.palette.bpp)
				pixeldatastr = "[%s]" % ", ".join(["0x%04x" % p for p in pixeldata])
				sprite_deflist.append("&%s_%d_%d" % (self.name, x, y))
				print(self.SPRITE_DEF % (self.name, x, y, pixeldatastr))

		sprite_deflist = ", ".join(sprite_deflist)
		print(self.SPRITE_MAP_DEF % (self.name,
				self.cnt_x * self.cnt_y,
				sprite_deflist,
				self.sprite_dim[0], self.sprite_dim[1],
				self.cnt_x, self.cnt_y,
				self.palette.name))

outfile = None
if len(sys.argv) == 2:
	outfile = open(sys.argv[1], "w")
	def print(s):
		outfile.write(s)
		outfile.write("\n")

pal_BW = Palette("PALETTE_BW", ["#000000", "#ffffff"])
pal_PTB = Palette("PALETTE_PTB", ["#000000", "#404040", "#808080", "#bfbfbf", "#ffffff", "#00adbc", "#55c8d2", "#aae3e7"])
pal_NITRO = Palette("PALETTE_NITRO", ["#000000", "#d1003b", "#e0557c", "#f0aabe", "#ffffff"])
pal_TEX = Palette("PALETTE_ICONS", ["#ffffff", "#c0c0c0", "#808080", "#000000", "#ff0000", "#b00000", "#00ff00", "#00b000", "#ffff00"])

img_font = Image("FONT_MAP", "font_9x18.png", pal_BW, (9, 18))
img_boldfont = Image("BOLD_FONT_MAP", "font_9x18_bold.png", pal_BW, (9, 18))
img_ptblogo = Image("PTB_LOGO", "ptb_logo.png", pal_PTB, (52, 15))
img_nitrologo = Image("NITRO_LOGO", "nitrokey_logo90.png", pal_NITRO, (30, 30))
img_texmap = Image("ICONS_MAP", "texmap.png", pal_TEX, (25, 10))
img_indicators = Image("INDICATOR_MAP", "indicators.png", pal_TEX, (10, 10))

img_font.emit()
img_boldfont.emit()
img_texmap.emit()
img_ptblogo.emit()
img_nitrologo.emit()
img_indicators.emit()

if outfile is not None:
	outfile.close()

sys.exit(0)
