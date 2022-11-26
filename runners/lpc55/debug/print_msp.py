import gdb


class WalkMSP(gdb.Command):

    def __init__(self):
        super(WalkMSP, self).__init__("walkmsp", gdb.COMMAND_USER)

    def invoke(self, args, from_tty):
        print(args)
        while True:
            try:
                # gdb.execute("f")
                gdb.execute("p/x $msp")
                gdb.execute("stepi 100")
            except KeyboardInterrupt:
                break
            except gdb.error:
                continue
            except Exception:
                pass


WalkMSP()
