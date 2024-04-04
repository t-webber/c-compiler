#define MACRO

#ifdef __UINTPTR_TYPE__
#ifdef MACRO
char x[] = "Should be present";
#else
char x[] = "Should NOT be present";
#endif /* Bonjour */
#endif /* Bonjour */

// #if !defined(_FILE_OFFSET_BITS) || _FILE_OFFSET_BITS != 64

// #include <include.h>
#include <stdio.h>

#ifdef MACRO
char z[] = "Should be present";
#else
char z[] = "Should NOT be present";
#endif /* Bonjour */
int remy = 10;
int a = 5;

int main(void) {
    char x[] = "Hello World!";
    printf("%s", x);
    return 0;
}
