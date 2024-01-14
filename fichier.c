#define MACRO

// #include <include.h>
#include <stdio.h>

#ifdef MACRO
char x[] = "Should be present";
#else
char x[] = "Should NOT be present";
#endif /* Bonjour */
int remy = 10;
int a = 5;

int main(void) {
    char x = "Hello World!";
    printf("%s", x);
    return 0;
}
