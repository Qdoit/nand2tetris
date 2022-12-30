// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Fill.asm

// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed. 
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.
    
    @color
    M=0
    @i
    M=0
(LOOP)
    @color
    M=-1
    @KBD
    D=M
    @NOTWHITE
    D;JNE
    @color
    M=0
(NOTWHITE)
    @color
    D=M
    @SCREEN
    D=A
    @i
    M=D
    (LOOP_INNER)
        @24576
        D=A
        @i
        D=D-M
        @LOOP_INNER_END
        D;JLE
        @color
        D=M
        @i
        A=M
        M=D
        @i
        M=M+1
        @LOOP_INNER
        0;JMP
    (LOOP_INNER_END)
    @LOOP
    0;JMP

(END)
@END
0;JMP    
