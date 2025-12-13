	.data
fmt_int: .asciz "%d\n"
fmt_str: .asciz "%s\n"

	.text
	.global main
main:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	sub sp, sp, #512
	mov x15, #5
	str x15, [sp, #0]
	mov x15, #6
	str x15, [sp, #8]
	ldr x15, [sp, #0]
	ldr x14, [sp, #8]
	add x15, x15, x14
	str x15, [sp, #16]
	mov x15, #5
	str x15, [sp, #0]
	mov x15, #6
	str x15, [sp, #8]
	ldr x15, [sp, #0]
	ldr x14, [sp, #8]
	add x15, x15, x14
	str x15, [sp, #16]
	mov x15, #5
	str x15, [sp, #0]
	mov x15, #6
	str x15, [sp, #8]
	ldr x15, [sp, #0]
	ldr x14, [sp, #8]
	add x15, x15, x14
	str x15, [sp, #16]
	mov x15, #5
	str x15, [sp, #0]
	mov x15, #6
	str x15, [sp, #8]
	ldr x15, [sp, #0]
	ldr x14, [sp, #8]
	add x15, x15, x14
	str x15, [sp, #16]
	mov x15, #5
	str x15, [sp, #0]
	mov x15, #6
	str x15, [sp, #8]
	ldr x15, [sp, #0]
	ldr x14, [sp, #8]
	add x15, x15, x14
	str x15, [sp, #16]
	mov x15, #5
	str x15, [sp, #0]
	mov x15, #6
	str x15, [sp, #8]
	ldr x15, [sp, #0]
	ldr x14, [sp, #8]
	add x15, x15, x14
	str x15, [sp, #16]
	mov x15, #5
	str x15, [sp, #0]
	mov x15, #6
	str x15, [sp, #8]
	ldr x15, [sp, #0]
	ldr x14, [sp, #8]
	add x15, x15, x14
	str x15, [sp, #16]
	mov x15, #5
	str x15, [sp, #0]
	mov x15, #6
	str x15, [sp, #8]
	ldr x15, [sp, #0]
	ldr x14, [sp, #8]
	add x15, x15, x14
	str x15, [sp, #16]
	mov x15, #5
	str x15, [sp, #0]
	mov x15, #6
	str x15, [sp, #8]
	ldr x15, [sp, #0]
	ldr x14, [sp, #8]
	add x15, x15, x14
	str x15, [sp, #16]
	mov x15, #5
	str x15, [sp, #0]
	mov x15, #6
	str x15, [sp, #8]
	ldr x15, [sp, #0]
	ldr x14, [sp, #8]
	add x15, x15, x14
	str x15, [sp, #16]
	add sp, sp, #512
	ldp x29, x30, [sp], #16
	mov x0, #0
	ret
