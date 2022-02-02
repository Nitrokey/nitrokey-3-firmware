#!/usr/bin/python3

import getopt
import sys

syms = {}
fp_mode = False
BEYOND_THRESHOLD = 0x01000000

opts, args = getopt.getopt(sys.argv[1:], "fl:t:")
types = "aAbBdDrRtT"

for o in opts:
	if o[0] == "-f":
		fp_mode = True
	elif o[0] == "-t":
		types = o[1]
	elif o[0] == "-l":
		BEYOND_THRESHOLD = int(o[1])

if len(args) == 0:
	raise ValueError("Please provide a symbol file name.")

fn = args[0]

with open(fn, "r") as f:
	while True:
		l = f.readline().strip()
		if l is None or l == "":
			break
		# print(l)
		la, lt, ln = l.split(" ", 2)
		if lt in types:
			syms[int(la, 16)] = ln

addrs = list(syms.keys())
addrs.sort()
print("%d symbols found, covering %08x--%08x." % (len(addrs), addrs[0], addrs[-1]))

def is_address(cand):
	try:
		if isinstance(cand, str):
			vi = int(cand, 16)
		else:
			vi = cand
		if vi > addrs[-1] + BEYOND_THRESHOLD or vi < addrs[0]:
			# sys.stderr.write("beyond last\n")
			return None
		addr0 = None
		off0 = None
		for a in addrs:
			if a <= vi and a + BEYOND_THRESHOLD > vi:
				addr0 = a
				off0 = vi - a
			elif a > vi:
				break
		if addr0 is None:
			# sys.stderr.write("none found\n")
			return None
		else:
			return (vi, addr0, off0)
	except:
		pass
	return None

def frame_at(maddr):
	sym = is_address(mem[maddr+4])
	if sym is None:
		return None
	return maddr, sym[0], sym[1], sym[2], mem[maddr]

mem = {}

for l in sys.stdin:
	lf = l.strip().split(" ")
	if fp_mode:
		lfa = int(lf[0].strip(":"), 16)
		for i in range(1, len(lf)):
			mem[lfa + 4*(i-1)] = int(lf[i], 16)
	else:
		for v in lf:
			if len(v) != 8:
				continue
			res = is_address(v)
			if res is not None:
				addr, addr0, off0 = res
				print("%08x\t%08x+%08x\t%s" % (addr, addr0, off0, syms[addr0]))

if fp_mode:
	memkeys = list(mem.keys())
	memkeys.sort()
	print("Analyzing memory dump %08x--%08x." % (memkeys[0], memkeys[-1]))
	for mv in memkeys:
		if mem[mv] > mv and mem[mv] < memkeys[-1] and is_address(mem[mv+4]) is not None:
			while True:
				frame = frame_at(mv)
				if frame is None:
					break
				print("@ %08x\tFP %08x\tLR %08x\t%08x+%08x\t%s" % (frame[0], frame[4], frame[1], frame[2], frame[3], syms[frame[2]]))
				mv = frame[4]
				if mv is None or not mv in mem:
					break
			break

sys.exit(0)
