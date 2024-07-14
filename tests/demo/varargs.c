#include "varargs.h"

#include <stdarg.h>
#include <stddef.h>

int add_ints(int first, ...) {
    int sum = first;
    va_list args;
    va_start(args, first);
 
    while (1) {
        int arg = va_arg(args, int);
        if (arg == 0) {
            break;
        }
        sum += arg;
    }
 
    va_end(args);
    return sum;
}
