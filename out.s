	.data
fmt_int: .asciz "%d\n"
fmt_str: .asciz "%s\n"
.LC0: .asciz "hihi"
.LC1: .asciz "haha"

	.text
	.global main
	main:
	stp x29, x30, [sp, #-16]!    // save fp and lr, push 16 bytes
	mov x29, sp                  // set frame pointer
	sub sp, sp, #512            // reserve 512 bytes for locals (simple stack frame)
	mov x15, #5        // literal 5
	str x15, [sp, #0]    // store 'x' into local slot
	mov x15, #6        // literal 6
	str x15, [sp, #8]    // store 'y' into local slot
	ldr x15, [sp, #0]    // load variable 'x'
	ldr x14, [sp, #8]    // load variable 'y'
	add x15, x15, x14    // x15 + x14
	str x15, [sp, #16]    // store 'z' into local slot
	ldr x15, [sp, #16]    // load variable 'z'
	mov x14, #0        // literal 0
	cmp x15, x14          // compare left and right
	cset x15, gt         // set x15 = (left > right) ? 1 : 0
	cmp x15, #0               // compare condition with 0
	beq else_0                 // if equal (false) branch to else
	adr x0, fmt_str         // load address of format string "%s\n"
	adr x1, .LC0             // address of the string literal
	bl printf                 // call printf(fmt_str, string)
	b end_if_1                    // jump to end_if after then
	else_0:                      // else label
	adr x0, fmt_str         // load address of format string "%s\n"
	adr x1, .LC1             // address of the string literal
	bl printf                 // call printf(fmt_str, string)
	end_if_1:                      // end_if label
	add sp, sp, #512            // deallocate frame
	ldp x29, x30, [sp], #16     // restore fp and lr
	mov x0, #0                  // return 0
	ret
