#!/bin/sh

# Take an image built merely with the right offsets (and possibly higher leve
# riotboot functionality such as reboot-into-bootloader), send it through
# riotboot-genhdr, and flash it using dfu-util.
#
# It may be practical to split this into a build-image-from-elf and a
# flash-image step using laze, but right now this works.

set -e

FLASH_OFFSET="$1"
FLASH_SLOT_OFFSET="$2"
FLASH_SIZE_TOTAL="$3"
FLASH_SLOT="$4"
FLASH_NUM_SLOTS="$5"
BUILT_ELF="$6"

IMAGE="${BUILT_ELF}.img"
VERSION=$(date +%s)

# FIXME: How do we best access this, if not by asking the user to get a RIOT
# checkout and place its dist/tools/riotboot_gen_hdr/bin/ in the PATH?
GENHDR=genhdr

objcopy "${BUILT_ELF}" -Obinary "${BUILT_ELF}.bin"

${GENHDR} generate \
    "${BUILT_ELF}.bin" \
    ${VERSION} \
    $((${FLASH_OFFSET} + (${FLASH_SIZE_TOTAL} - ${FLASH_OFFSET}) / ${FLASH_NUM_SLOTS} * ${FLASH_SLOT} + ${FLASH_SLOT_OFFSET})) \
    ${FLASH_SLOT_OFFSET} \
    "${BUILT_ELF}.riotboot"

cat "${BUILT_ELF}.riotboot" "${BUILT_ELF}.bin" > "${IMAGE}"

dfu-util --device 1209:7d02 --alt ${FLASH_SLOT} --download "${IMAGE}"
