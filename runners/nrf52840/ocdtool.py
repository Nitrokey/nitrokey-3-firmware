#!/usr/bin/python3

import socket
import subprocess
import sys

if len(sys.argv) != 2:
	sys.exit(1)

if "D" in sys.argv[1]:
	BOARD = "nrfdk"
elif "P" in sys.argv[1]:
	BOARD = "proto1"
elif "N" in sys.argv[1]:
    BOARD = "nk3mini"
else:
	sys.exit(1)

SYMBOL_FILE = ("symbols-%s-release.txt" % BOARD)

OP_HALT = "h" in sys.argv[1]
OP_RTT = "r" in sys.argv[1]
OP_FRAMEDUMP = "f" in sys.argv[1]

s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.connect(('127.0.0.1', 4444))

def read_up(sock):
	s = b""
	while len(s) < 2 or not s.endswith(b"> "):
		s_ = sock.recv(4096)
		if len(s_) == 0:
			return s, None
		s += s_
	data = s.replace(b"\r", b"").replace(b"\x00", b"").split(b"\n")
	return data[0:-1], b"> "

def command(sock, cmd):
	cmd_ = cmd.encode("ascii") + b"\r\n"
	l = sock.send(cmd_)
	assert l == len(cmd_)
	return read_up(sock)

def get_register(sock, reg):
	d, p = command(sock, ("reg %s" % reg))
	vals = d[1].decode().split(" ")
	assert vals[0] == reg
	assert vals[2].startswith("0x")
	return int(vals[2], 16)

def get_memory(sock, base, size):
	d, p = command(sock, ("mdw 0x%08x %d" % (base, size//4)))
	mem = {}
	for dumpline in d[1:]:
		fields = dumpline.decode().strip().split(" ")
		if len(fields) <= 1:
			continue
		assert fields[0].endswith(":")
		laddr = int(fields[0].strip(":"), 16)
		for i in range(len(fields)-1):
			v = int(fields[i+1], 16)
			assert (laddr + (i*4)) not in mem
			mem[laddr + (i*4)] = v
	return mem

def emit_frame(s, pc, sp, fp):
	fsz = fp - sp
	print("%.8s PC %08x SP %08x FP %08x (SZ % 5x)" % (s, pc, sp, fp, fsz))

data, prompt = read_up(s)

if OP_HALT:
	command(s, "halt")

if OP_RTT:
	rtt_base = 0x20000000
	top_of_ram = 0x20040000
	with open(SYMBOL_FILE, "r") as symf:
		for symline in symf:
			symfields = symline.strip().split(" ")
			if symfields[2] == "_SEGGER_RTT":
				rtt_base = int(symfields[0], 16)
				break
	command(s, "rtt setup 0x%08x %d \"SEGGER RTT\"" % (rtt_base, top_of_ram - rtt_base))
	command(s, "rtt start")
	command(s, "rtt server start 4999 0")

if not OP_FRAMEDUMP:
	sys.exit(0)

stacktrace = []
# print(data, prompt)
sp = get_register(s, "sp")
fp = get_register(s, "r7")
pc = get_register(s, "pc")
emit_frame("BREAK AT", pc, sp, fp)
stacktrace.append(pc)

while fp > 0x20000000 and fp < 0x20040000:
	if (pc & 0xffffffe0) == 0xffffffe0:
		m = get_memory(s, sp, 8*4)
		pc = m[sp+6*4]
		sp = sp + 8*4
		# fp stays
		emit_frame("\-iFRAME", pc, sp, fp)
	else:
		m = get_memory(s, fp, 2*4)
		sp = fp+8
		pc = m[fp+4]
		fp = m[fp]
		if (pc & 0xffffffe0) == 0xffffffe0:
			xfp = sp + 8*4
		else:
			xfp = fp
		emit_frame("\- FRAME", pc, sp, xfp)
	stacktrace.append(pc)

s.close()

syp = subprocess.Popen("python syms.py -t Tt %s" % SYMBOL_FILE, stdin=subprocess.PIPE, stdout=subprocess.PIPE, shell=True, text=True)
syp_in = "".join([("%08x\n" % s) for s in stacktrace])
syp_out, syp_err = syp.communicate(syp_in)
assert syp.returncode == 0
print("RESOLVED BACKTRACE:")
print(syp_out)

sys.exit(0)
