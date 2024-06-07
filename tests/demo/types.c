#include "types.h"

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum TaggedUnionType {
	kTaggedUnionSize,
	kTaggedUnionDouble,
	kTaggedUnionString,
};

struct TaggedUnion {
	enum TaggedUnionType type;
	union {
		size_t size;
		double double_val;
		char* string;
	} value;
	size_t number;
	char* (*format_fn)(struct TaggedUnion*);
	size_t name_len;
	char name[];
};

static size_t tagged_union_index() {
	static size_t index = 0;
	return index++;
}

static char* allocating_sprintf(char* format, ...) {
	va_list args, args2;
	va_start(args, format);
	va_copy(args2, args);
	size_t size = vsnprintf(NULL, 0, format, args) + 1;
	char* ret = malloc(size);
	vsnprintf(ret, size, format, args2);
	return ret;
}

static char* default_format_fn(struct TaggedUnion* value) {
	switch (value->type) {
		case kTaggedUnionSize:
			return allocating_sprintf(
					"TaggedUnion(type = size_t, name = '%s', index = %zu, value = %zu)",
					value->name, value->number, value->value.size);
		case kTaggedUnionDouble:
			return allocating_sprintf(
					"TaggedUnion(type = double, name = '%s', index = %zu, value = %f)",
					value->name, value->number, value->value.double_val);
		case kTaggedUnionString:
			return allocating_sprintf(
					"TaggedUnion(type = string, name = '%s', index = %zu, value = '%s')",
					value->name, value->number, value->value.string);
		default:
			abort();
	}
}

static struct TaggedUnion* allocate_tagged_union_common(char* name) {
	size_t string_len = strlen(name);
	size_t size = sizeof(struct TaggedUnion) + string_len + 1;
	struct TaggedUnion* tagged = malloc(size);

	tagged->number = tagged_union_index();
	tagged->format_fn = default_format_fn;
	tagged->name_len = string_len;
	strncpy(tagged->name, name, string_len);

	return tagged;
}

struct TaggedUnion* allocate_tagged_union_size(size_t value, char* name) {
	struct TaggedUnion* tagged = allocate_tagged_union_common(name);
	tagged->type = kTaggedUnionSize;
	tagged->value.size = value;
	return tagged;
}

struct TaggedUnion* allocate_tagged_union_double(double value, char* name) {
	struct TaggedUnion* tagged = allocate_tagged_union_common(name);
	tagged->type = kTaggedUnionDouble;
	tagged->value.double_val = value;
	return tagged;
}

struct TaggedUnion* allocate_tagged_union_str(char* value, char* name) {
	struct TaggedUnion* tagged = allocate_tagged_union_common(name);
	tagged->type = kTaggedUnionString;
	tagged->value.string = value;
	return tagged;
}

char* tagged_union_format(struct TaggedUnion* tag) {
	return tag->format_fn(tag);
}

void free_tagged_union(struct TaggedUnion* tag) {
	free(tag);
}
