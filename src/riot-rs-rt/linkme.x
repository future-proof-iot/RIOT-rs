SECTIONS {
  linkme_INIT_FUNCS : { *(linkme_INIT_FUNCS) } > FLASH
  linkm2_INIT_FUNCS : { *(linkm2_INIT_FUNCS) } > FLASH
  linkme_EMBASSY_TASKS : { *(linkme_EMBASSY_TASKS) } > FLASH
  linkm2_EMBASSY_TASKS : { *(linkm2_EMBASSY_TASKS) } > FLASH
}

INSERT AFTER .rodata
