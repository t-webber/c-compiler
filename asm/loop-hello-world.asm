    global _main
    extern _GetStdHandle@4
    extern _WriteFile@20
    extern _ExitProcess@4

    section .data 
increment db 0

    section .text
_main:
    ; DWORD  bytes;    
    mov ebp, esp
    sub esp, 4

    ; hStdOut = GetstdHandle( STD_OUTPUT_HANDLE)
    push -11
    call _GetStdHandle@4
    mov  ebx, eax

    StartLoop:
    ; mov  ebx, 4
    mov eax, [increment]
    cmp al, 5
    jge EndLoop

        ; WriteFile( hstdOut, message, length(message), &bytes, 0);
    push 0
    lea  eax, [ebp-4]
    push eax
    
    push (message_end - message)
    push message
    push ebx
    call _WriteFile@20

    inc byte [increment]

    jmp StartLoop

    EndLcoop:

    ; ExitProcess(0)
    push 0
    call _ExitProcess@4

    ; never here
    hlt
message:
    db 'Hello, World!', 10
message_end: