SECTIONS {
  linkme_INIT_FUNCS : { *(linkme_INIT_FUNCS) } > FLASH
  linkm2_INIT_FUNCS : { *(linkm2_INIT_FUNCS) } > FLASH
  linkme_EMBASSY_TASKS : { *(linkme_EMBASSY_TASKS) } > FLASH
  linkm2_EMBASSY_TASKS : { *(linkm2_EMBASSY_TASKS) } > FLASH
  linkme_USB_BUILDER_HOOKS : { *(linkme_USB_BUILDER_HOOKS) } > FLASH
  linkm2_USB_BUILDER_HOOKS : { *(linkm2_USB_BUILDER_HOOKS) } > FLASH
  linkme_THREAD_FNS : { *(linkme_THREAD_FNS) } > FLASH
  linkm2_THREAD_FNS : { *(linkm2_THREAD_FNS) } > FLASH
}

INSERT AFTER .rodata
