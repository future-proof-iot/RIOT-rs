SECTIONS
{
    . = ALIGN(8);
    .data :
    {
        KEEP (*(SORT(.xfa.*)))
    } > RAM AT>FLASH
}

INSERT AFTER .data;

SECTIONS
{
    .rodata : ALIGN(8)
    {
        KEEP (*(SORT(.roxfa.*)))
    } > FLASH
}

INSERT AFTER .text ;

