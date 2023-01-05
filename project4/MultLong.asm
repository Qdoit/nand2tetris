// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Mult.asm

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)
//
// This program only needs to handle arguments that satisfy
// R0 >= 0, R1 >= 0, and R0*R1 < 32768.

// The code here implements long multiplication, which is much more efficient for larger numbers.
// Right shift is implemented manually
    @R2 
    M=0
    
(LOOP)
    @R1
    D=M
    @END
    D;JEQ
    
    @R1
    D=M
    @1
    D=D&A
    @LAST_IS_ZERO
    D;JEQ
        @R0
        D=M
        @R2
        M=M+D
    (LAST_IS_ZERO)
    
    @R0
    D=M
    M=M+D

    @SHIFT_RIGHT
    0;JMP
(shift_right_callback)
    
    @LOOP
    0; JMP

(END)
    @END
    0;JMP

// Right shifts R1
// callback in @shift_right_callback
(SHIFT_RIGHT)
    @157
    D=A
    @mdbg
    M=D
    @sri
    M=0
    @mask1
    M=1
    @2
    D=A
    @mask2
    M=D
    @R1
    D=M
    @divtemp
    M=D
    @R1
    M=0
    (SR_LOOP)
    @sri
    D=M
    @15
    D=A-D
    @SR_LOOP_END
    D;JEQ
        @mdbg
        M=0
        @divtemp
        D=M
        @mask2
        D=D&M
        @SR_JUMP_1
        D;JEQ
            @mdbg
            M=1
            @mask1
            D=M
            @R1
            M=M|D
        (SR_JUMP_1)
        
        @mask1
        D=M
        M=D+M
        @mask2
        D=M
        M=D+M
    @1
    D=A
    @sri
    M=M+D
    @SR_LOOP
    0;JMP
    (SR_LOOP_END)
@shift_right_callback
0; JMP

