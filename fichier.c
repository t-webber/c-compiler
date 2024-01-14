#define MACRO "Should bepresent"
#define OTHER "toto"

#if MACRO == OTHER
char x[] = "Should be present"
#else
char x[] = "Should NOT2 be present";
#endif /* Bonjour */
int remy = 10;
int a = 5;

int main(void) {
    char x = "Hello Worldy!";
    printf("%s", x);
    return 0;
}
