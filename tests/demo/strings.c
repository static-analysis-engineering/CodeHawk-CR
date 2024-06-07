#include "strings.h"

#include <string.h>
#include <stdio.h>

void print_twice(char* str) {
	printf("%s", str);
	printf("%s", str);
}

void snprintf_twice(char* in, char* out, size_t out_len) {
	size_t written = snprintf(out, out_len, "%s", in);
	if (out_len > written) {
		out += written;
		out_len -= written;
		snprintf(out, out_len, "%s", in);
	}
}
