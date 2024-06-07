#pragma once

#include <stdlib.h>

struct TaggedUnion;

struct TaggedUnion* allocate_tagged_union_size(size_t value, char* name);
struct TaggedUnion* allocate_tagged_union_double(double value, char* name);
struct TaggedUnion* allocate_tagged_union_str(char* value, char* name);

char* tagged_union_format(struct TaggedUnion*);

void free_tagged_union(struct TaggedUnion*);
