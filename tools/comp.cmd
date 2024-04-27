cls
del test.obj
del test.exe
nasm -f win32 test.asm
call ..\lib\link.exe test.obj "..\lib\kernel32.lib" /ENTRY:main
call test.exe