MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 4K - 0x100
    STATE : ORIGIN = 0x101FF000, LENGTH = 4K
    RAM   : ORIGIN = 0x20000000, LENGTH = 264K
}

SECTIONS {
    .boot2 : {
        KEEP(*(.boot2));
    } > BOOT2
} INSERT BEFORE .text;

SECTIONS {
  .app_state : {
    KEEP(*(.app_state))
  } > STATE
}