#if 1+2==2
#if 3*5+2
#if 3*6-4-2
// Marche pas en dessous
#define MACRO 2
#if defined MACRO
// Plante en dessous
#if !+(2+3)
#if 1++
int a = 5;

int main(void) {
    char x[] = "Hello World!";
    printf("%s", x);
    return 0;
}