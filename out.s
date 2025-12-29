	.data
fmt_int: .asciz "%d\n"
fmt_str: .asciz "%s\n"
fmt_scan: .asciz "%s"
msg_pay: .asciz "free trial over pew pew, type 'haha' to continue: "
secret:  .asciz "haha"

	.text
	.global main
main:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	sub sp, sp, #512
	bl rand
	and x14, x0, #1
	cmp x14, #0
	beq else_0
	ldr x14, [sp, #0]
	adrp x0, fmt_int
	add  x0, x0, :lo12:fmt_int
	mov x1, x14
	bl printf
	b endif_1
else_0:
	ldr x14, =69
	adrp x0, fmt_int
	add  x0, x0, :lo12:fmt_int
	mov x1, x14
	bl printf
endif_1:
	adrp x0, msg_pay
	add x0, x0, :lo12:msg_pay
	bl printf
	adrp x0, fmt_scan
	add x0, x0, :lo12:fmt_scan
	add x1, sp, #400
	bl scanf
	add x0, sp, #400
	adrp x1, secret
	add x1, x1, :lo12:secret
	bl strcmp
	cmp x0, #0
	beq paid_2
	mov x0, #1
	mov x8, #93
	svc #0
paid_2:
	ldr x14, =999999
	adrp x0, fmt_int
	add  x0, x0, :lo12:fmt_int
	mov x1, x14
	bl printf
	add sp, sp, #512
	ldp x29, x30, [sp], #16
	mov x0, #0
	ret
