// test

// count down
LDI r1 10
.loop // I'm a label on my own line
DEC r1
BRH zero .exit
JMP .loop
.exit HLT // I'm a label on the same line as an instruction
