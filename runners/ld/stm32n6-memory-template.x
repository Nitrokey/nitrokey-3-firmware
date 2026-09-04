MEMORY
{
  /* AXISRAM1, loaded by the debugger in development boot mode */
  FLASH : ORIGIN = ##FLASH_BASE##, LENGTH = ##FLASH_LENGTH##

  /* AXISRAM2 */
  RAM : ORIGIN = 0x34100000, LENGTH = 1024K
}

/* flip-link places the stack below .data, so it ends at the start of RAM (used for MSPLIM). */
_stack_end = 0x34100000;
