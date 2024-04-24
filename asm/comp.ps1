clear-host
remove-item test.obj
remove-item test.exe
nasm -f win32 test.asm
& $PSScriptRoot/../lib/link.exe test.obj "$PSScriptRoot/../lib/kernel32.lib" /ENTRY:main
& "$PSScriptRoot/test.exe"
