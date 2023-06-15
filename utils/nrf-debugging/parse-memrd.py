
import sys


data = []

for line in open(sys.argv[1]):
    _data = []
    #if not line: continue
    if ":" not in line: continue
    line = line.split(":")[1]
    line = line.split("|")[0].strip().replace(" ", "")
    #print(len(line), line.strip())
    if len(line) == 0: continue
    for idx in range(0, 32, 2):

        a, b = line[idx], line[idx+1]
        x = "0x"+a+b
        _data.append(int(x, 16))

    data.extend([_data[3], _data[2], _data[1], _data[0]])
    data.extend([_data[7], _data[6], _data[5], _data[4]])
    data.extend([_data[11], _data[10], _data[9], _data[8]])
    data.extend([_data[15], _data[14], _data[13], _data[12]])



bdata = bytes(data)

with open("output.bin", "wb") as fd:
    fd.write(bdata)

