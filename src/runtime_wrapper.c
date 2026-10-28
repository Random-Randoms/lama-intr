#include <stddef.h>

/// Stub definitions for Lama runtime
void* __start_custom_data;
void* __stop_custom_data;

/// Operand stack (with Lama gc awareness)
extern size_t __gc_stack_top, __gc_stack_bottom;

void set_stack_borders(size_t top, size_t bottom) {
    __gc_stack_top = top;
    __gc_stack_bottom = bottom;
}
