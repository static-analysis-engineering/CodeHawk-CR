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
//! Variable definition.

use pyo3::prelude::*;

use crate::{
    app::{c_declarations::CDeclarations, c_dictionary_record::CDeclarationsRecord},
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_var_info")?;
    module.add_class::<CVarInfo>()?;
    Ok(module)
}

/// Local or global variable.
///
/// * tags[0]: vname
/// * tags[1]: vstorage  ('?' for global variable, 'o_gvid' for opaque variable)
///
/// * args[0]: vid       (-1 for global variable)
/// * args[1]: vtype
/// * args[2]: vattr     (-1 for global variable) (TODO: add global attributes)
/// * args[3]: vglob
/// * args[4]: vinline
/// * args[5]: vdecl     (-1 for global variable) (TODO: add global locations)
/// * args[6]: vaddrof
/// * args[7]: vparam
/// * args[8]: vinit     (optional)
#[derive(Clone)]
#[pyclass(extends = CDeclarationsRecord, frozen, subclass)]
pub struct CVarInfo {}

#[pymethods]
impl CVarInfo {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDeclarationsRecord::new(cd, ixval)).add_subclass(CVarInfo {})
    }
}
