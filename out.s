	.data
fmt_int: .asciz "%d\n"
fmt_str: .asciz "%s\n"
.LC0: .asciz "69"
.LC1: .asciz "420"

	.text
	.global main
main:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	sub sp, sp, #512
	adrp x15, .LC0
	add  x15, x15, :lo12:.LC0
	str x15, [sp, #0]
	adrp x15, .LC1
	add  x15, x15, :lo12:.LC1
	str x15, [sp, #8]
	ldr x15, [sp, #0]
	ldr x14, [sp, #8]
	cmp x15, x14
	cset x15, gt
	cmp x15, #0
	beq else_0
	ldr x14, [sp, #0]
	ldr x13, [sp, #8]
	add x14, x14, x13
	adrp x0, fmt_int
	add  x0, x0, :lo12:fmt_int
	mov x1, x14
	bl printf
	b endif_1
else_0:
	ldr x14, [sp, #8]
	ldr x13, [sp, #0]
	sub x14, x14, x13
	adrp x0, fmt_int
	add  x0, x0, :lo12:fmt_int
	mov x1, x14
	bl printf
endif_1:
	add sp, sp, #512
	ldp x29, x30, [sp], #16
	mov x0, #0
	ret
