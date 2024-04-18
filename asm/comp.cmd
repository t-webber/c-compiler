cls
del test.obj
del test.exe
nasm -f win32 test.asm
LINK test.obj "D:\Windows Kits\10\Lib\10.0.22621.0\um\x86\kernel32.Lib" /ENTRY:main
test.exe
