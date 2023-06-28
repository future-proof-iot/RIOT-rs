SECTIONS {
  linkme_INIT_FUNCS : { *(linkme_INIT_FUNCS) } > FLASH
  linkm2_INIT_FUNCS : { *(linkm2_INIT_FUNCS) } > FLASH
}

INSERT AFTER .rodata
