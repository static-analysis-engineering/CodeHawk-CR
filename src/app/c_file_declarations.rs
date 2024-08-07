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
use pyo3::{intern, prelude::*};

use crate::{
    app::{c_declarations::CDeclarations, c_type_info::CTypeInfo},
    util::indexed_table::IndexedTable,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_file_declarations")?;
    module.add_class::<CFileDeclarations>()?;
    Ok(module)
}

/// C File level definitions and declarations.
///
/// This information is originally written by cchcil/cHCilWriteXml:
/// write_xml_cfile based on cchcil/cHCilDeclarations.cildeclarations.
///
/// Declarations are dependent on CFileDictionary
#[pyclass(extends = CDeclarations, frozen, subclass)]
pub struct CFileDeclarations {}

#[pymethods]
impl CFileDeclarations {
    #[new]
    fn new() -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDeclarations::new()).add_subclass(CFileDeclarations {})
    }

    fn get_typeinfo<'a>(slf: &Bound<'a, Self>, ix: isize) -> PyResult<Bound<'a, CTypeInfo>> {
        let py = slf.py();
        let itv = slf
            .getattr(intern!(py, "typeinfo_table"))?
            .downcast::<IndexedTable>()?
            .clone()
            .borrow()
            .retrieve(ix)?
            .get()
            .clone();
        Bound::new(
            py,
            CTypeInfo::new(slf.downcast::<CDeclarations>()?.clone().unbind(), itv),
        )
    }
}
