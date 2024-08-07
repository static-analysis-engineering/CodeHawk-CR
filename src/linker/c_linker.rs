/*
------------------------------------------------------------------------------
CodeHawk C Analyzer
Author: Henny Sipma
------------------------------------------------------------------------------
The MIT License (MIT)

Copyright (c) 2017-2020 Kestrel Technology LLC
Copyright (c) 2020-2022 Henny B. Sipma
Copyright (c) 2023-2024 Aarno Labs LLC

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
------------------------------------------------------------------------------
*/
/// Starting point: a list of (fileindex,compinfo key) pairs that identify the
///    locally declared structs
///
/// Goal: produce equivalence classes of (fileindex,compinfo key) pairs that
///    are associated with (structurally) equivalent structs, assign a
///    global id to each distinct struct, and create a mapping between the
///    (fileindex,compinfo key) pairs and the global id (xrefs) and a
///    mapping between the global id and an instance of a struct from the
///    corresponding equivalence class. All nested field struct types must
///    be renamed with global ids.
use pyo3::prelude::*;

use crate::app::c_application::CApplication;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_linker")?;
    module.add_class::<CLinker>()?;
    Ok(module)
}

#[pyclass(get_all, subclass)]
pub struct CLinker {
    capp: Py<CApplication>,
}

#[pymethods]
impl CLinker {
    #[new]
    fn new(capp: Py<CApplication>) -> CLinker {
        CLinker { capp }
    }
}
