#undef TOTO
#define   TOTO    (1+2)

 #define TOTO (x) // # define TOTO (x) ""
//  #define TOTO (1) // = #define TOTO 1
#define TOTO2 ((1+2)*3)
#define TOTO3   ( 1 * ( (2) +  (( 3))  ) )   
#undef TOTO2
#define TOTO4  ( x )  ( x * ( (2) +  (( 3))  ) )   
 
int main(void) {
    printf("%s", &TOTO);
    return 0;
}