param(
    [string]$name = "test"
)
clear-host

if ($name -like "*.asm") {
    $name = $name.Substring(0, $name.Length - 4)
}

remove-item "$name.obj" -erroraction silentlycontinue
remove-item "$name.exe" -erroraction silentlycontinue
nasm -f win32 "$name.asm"
& $psscriptroot/../lib/link.exe "$name.obj" "$psscriptroot/../lib/kernel32.lib" /entry:main /out:"$name.exe"
if (test-path "$name.exe") {
    & "$name.exe" 
    write-host "--- success ---" -foregroundcolor green
}
else {
    write-host "--- failed ---" -foregroundcolor red
}
