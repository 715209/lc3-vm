.Orig x3000
INIT_CODE
LEA R6, #-1
ADD R5, R6, #0
ADD R6, R6, R6
ADD R6, R6, R6
ADD R6, R6, R5
ADD R6, R6, #-1
ADD R5, R5, R5
ADD R5, R6, #0
LD R4, GLOBAL_DATA_POINTER

GLOBAL_DATA_POINTER .FILL GLOBAL_DATA_START

ADD R6, R6, #-2
STR R7, R6, #0
ADD R6, R6, #-1
STR R5, R6, #0
ADD R5, R6, #-1

ADD R6, R6, #-2
ADD R7, R4, #7
ldr R7, R7, #0
str R7, R5, #-1
ADD R7, R4, #7
ldr R7, R7, #0
str R7, R5, #0
ADD R0, R4, #2
LDR R0, R0, #0
JMP R0
lc3_L3_test
ldr R7, R5, #0
ADD R3, R4, #6
ldr R3, R3, #0
add R7, R7, R3
str R7, R5, #0
ldr R7, R5, #0
ADD R3, R4, #5
ldr R3, R3, #0
NOT R7, R7
ADD R7, R7, #1
ADD R7, R7, R3
BRz L11
ADD R7, R4, #1
LDR R7, R7, #0
jmp R7
L11
ldr R7, R5, #-1
ADD R3, R4, #6
ldr R3, R3, #0
add R7, R7, R3
str R7, R5, #-1
ADD R7, R4, #7
ldr R7, R7, #0
str R7, R5, #0
lc3_L7_test
lc3_L4_test
ldr R7, R5, #-1
ADD R3, R4, #4
ldr R3, R3, #0
NOT R7, R7
ADD R7, R7, #1
ADD R7, R7, R3
BRnz L12
ADD R7, R4, #0
LDR R7, R7, #0
jmp R7
L12
ldr R7, R5, #-1
lc3_L1_test
STR R7, R5, #3
ADD R6, R5, #1
LDR R5, R6, #0
ADD R6, R6, #1
LDR R7, R6, #0
ADD R6, R6, #1
HALT

GLOBAL_DATA_START
L3_test .FILL lc3_L3_test
L7_test .FILL lc3_L7_test
L4_test .FILL lc3_L4_test
L1_test .FILL lc3_L1_test
L10_test .FILL #10         ; change this to count more
L9_test .FILL #32768
L6_test .FILL #1
L2_test .FILL #0
.END
