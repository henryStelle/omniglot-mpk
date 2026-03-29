#include "add.h"
#include <stdlib.h>

struct aa {
    char abc;
    char one;
    char four;
    char five;
};

// works
int *add_ptr(int a, int b) {
    int* mem = malloc(sizeof(int));
    *mem = a + b;
    return mem;
}

// works
int add(int a, int b) {
    return a + b;
}

// this segfaults
int* cannot_deref_ptr_add(int* a, int b) {
    *a = *a + b;
    return a;
}
