.global _start
.align 2

.section .rodata            // Read-only data section
message:
    .ascii "Hello,ARM64!\n" // The string to print
    message_len = . - message // Calculate length

.section .text              // Code section
_start:
    // Write syscall (print to stdout)
    mov x0, #1              // x0 = file descriptor (1 = stdout)
    ldr x1, =message        // x1 = pointer to message
    mov x2, #message_len    // x2 = length of message
    mov x8, #64             // x8 = syscall number (64 = write)
    svc #0                  // Make the syscall
    
    // Exit syscall
    mov x0, #0              // x0 = exit code (0 = success)
    mov x8, #93             // x8 = syscall number (93 = exit)
    svc #0                  // Make the syscall
    


        .data
fmt_str:    .asciz "%s\n"
hi_str:     .asciz "hi world"
bye_str:    .asciz "bye world"

    .text
    .global main
main:
    // prologue
    stp x29, x30, [sp, #-16]!
    mov x29, sp

    sub sp, sp, #32

    // x = 5;
    mov x9, #5
    str x9, [sp, #0]

    // y = 6;
    mov x9, #6
    str x9, [sp, #8]

    // z = x + y;
    ldr x10, [sp, #0]
    ldr x11, [sp, #8]
    add x12, x10, x11
    str x12, [sp, #16]

    // if (z > 10)
    ldr x13, [sp, #16]
    mov x14, #10
    cmp x13, x14
    b.le else_label

then_label:
    adr x0, fmt_str
    adr x1, hi_str
    bl printf
    b end_if

else_label:
    adr x0, fmt_str
    adr x1, bye_str
    bl printf

end_if:
    add sp, sp, #32
    ldp x29, x30, [sp], #16
    mov x0, #0
    ret
