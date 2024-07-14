#include "constants.h"
#include "strings.h"
#include "types.h"
#include "varargs.h"

int main() {
	char data[8];
	snprintf_twice("hello", data, sizeof(data));
	print_twice(data);
	printf("\n%s\n", kConstantStr);

	struct TaggedUnion* tags[3];
	tags[0] = allocate_tagged_union_size(5, "tag0");
	tags[1] = allocate_tagged_union_double(0.5, "tag1");
	tags[2] = allocate_tagged_union_str("mystring", "tag0");

	for (size_t i = 0; i < 3; i++) {
		char* formatted = tagged_union_format(tags[i]);
		printf("%s\n", formatted);
		free(formatted);
	}

	for (size_t i = 0; i < 3; i++) {
		free_tagged_union(tags[i]);
		tags[i] = NULL;
	}

	printf("%d\n", add_ints(1, 2, 3, 4, 5, 0));
}
