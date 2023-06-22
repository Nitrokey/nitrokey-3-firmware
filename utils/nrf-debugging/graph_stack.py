from matplotlib import pyplot as plt

"""
Quickly graph gdb msp prints.

rg -F "= (*mut ()) " gdb.txt > gdb.txt.stack

Expected format is like:

...
$38057 = (*mut ()) 0x2001b478
$38058 = (*mut ()) 0x2001b478
...

"""

PATH = 'gdb.txt.stack'

def main():
    data = []
    with open(PATH, 'r') as f:
        for line in f:
            num = int(line.split()[-1], 16)
            num = num - 0x20000000
            num = num // 1024
            data.append(num)

    print(len(data))
    plt.plot(data)
    plt.ylabel('free stack memory left [kB]')
    plt.xlabel('sample no.')
    plt.show()


if __name__ == '__main__':
    main()
