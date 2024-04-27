    global _main
    extern _GetStdHandle@4
    extern _WriteFile@20

    section .text
_main:
    ; hStdOut = GetstdHandle( STD_OUTPUT_HANDLE)
    push -11
    call _GetStdHandle@4
    mov  ebx, eax

    ; WriteFile( hstdOut, message, length(message), &bytes, 0);
    push 0
    lea  eax, [ebp]
    push eax
    push 3
    push message
    push ebx
    call _WriteFile@20

    section .data
message: db 'Hey'