Rough steps for migration, with a lot of overlap possible between steps:

1. Increase test coverage.

1. Convert `__init__` constructors to `__new__` constructors, because of a
   [pyo3 limitation](https://pyo3.rs/main/class#constructor).

1. Place a Rust superclass over every Python class.

1. Move instance variables from the Python constructor into the Rust class.

1. Convert individual methods of Python classes into Rust. When migrating a
   caller before a callee or when invoking a function that is overridden in a
   subclass, use pyo3 Python function invocation. Use `Bound` psuedo-receivers
   to access superclasses.

1. Duplicate constant data from superclasses into subclasses.

1. Use `&self` method receivers.

1. *Replace polymorphic superclasses with traits.*

1. *Replace all pyo3 Python calls with Rust calls.*
