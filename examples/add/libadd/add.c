#include "add.h"
#include <stdint.h>
#include <stdlib.h>

int add(int a, int b) { return a + b; }
// works

// works
int *add_ptr(int a, int b) {
        int *mem = malloc(sizeof(int));
        *mem = a + b;
        return mem;
}

// this segfaults
int *cannot_deref_ptr_add(int *a, int b) {
        *a = *a + b;
        return a;
}

// evil
/* int *evil_cannot_deref_ptr_add(int *a, int b) { */
/*     uint32_t old_pkru; */
/*     uint32_t ecx = 0; */
/*     uint32_t edx = 0; */

/*     // 1. READ the current PKRU state and save it */
/*     __asm__ volatile("rdpkru" : "=a"(old_pkru) : "c"(ecx) : "rdx"); */

/*     // 2. MODIFY: Clear bits 6 and 7 (to allow Access/Write for PKey 3) */
/*     uint32_t new_pkru = 0x0; */

/*     // 3. WRITE the new PKRU state to the CPU */
/*     __asm__ volatile("wrpkru" : : "a"(new_pkru), "c"(ecx), "d"(edx) :
 * "memory"); */

/*     // --- CRITICAL SECTION: The actual work --- */
/*     *a = *a + b; */
/*     // ----------------------------------------- */

/*     // 4. RESTORE the original PKRU state before returning */
/*     __asm__ volatile("wrpkru" : : "a"(old_pkru), "c"(ecx), "d"(edx) :
 * "memory"); */

/*     return a; */
/* } */

int *evil_cannot_deref_ptr_add(int *a, int b) {
        uint32_t old_pkru;

        // save current state
        __asm__ volatile("rdpkru" : "=a"(old_pkru) : "c"(0) : "rdx");

        // OPEN EVERYTHING (set PKRU to 0)
        __asm__ volatile("wrpkru" : : "a"(0), "c"(0), "d"(0) : "memory");

        *a = *a + b;

        // restore pkey
        __asm__ volatile("wrpkru" : : "a"(old_pkru), "c"(0), "d"(0) : "memory");

        return a;
}

// works
int add_to_callback(int a, int (*callback)(int)) {
        int callback_result = callback(a);
        return a + callback_result;
}
