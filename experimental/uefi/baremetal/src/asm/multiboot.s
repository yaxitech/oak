.section .multiboot_header, "a"
.code32
.align 4

MULTIBOOT_HEADER_MAGIC = 0x1BADB002
MULTIBOOT_HEADER_FLAGS = (1 << 1)

multiboot_header:
    .long MULTIBOOT_HEADER_MAGIC
    .long MULTIBOOT_HEADER_FLAGS
    .long -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS)