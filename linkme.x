SECTIONS {
  .linkme :
  {
    KEEP(*(SORT(.linkme.*)));
  } > FLASH
}

INSERT AFTER .rodata
