nasm -f elf64 program.asm -o program.o
gcc -c runtime.c -o runtime.o
gcc program.o runtime.o -o program
