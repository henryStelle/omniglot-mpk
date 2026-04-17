int add(int a, int b);
int* add_ptr(int a, int b);

int* cannot_deref_ptr_add(int* a, int b);
int* evil_cannot_deref_ptr_add(int* a, int b);
int add_to_callback(int a, int (*callback)(int));
