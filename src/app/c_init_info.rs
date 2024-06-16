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
use pyo3::prelude::*;

use crate::{
    app::{c_declarations::CDeclarations, c_dictionary_record::CDeclarationsRecord},
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_init_info")?;
    module.add_class::<CCompoundInitInfo>()?;
    module.add_class::<CInitInfo>()?;
    module.add_class::<CSingleInitInfo>()?;
    module.add_class::<COffsetInitInfo>()?;
    Ok(module)
}

/// Global variable initializer.
#[derive(Clone)]
#[pyclass(extends = CDeclarationsRecord, frozen, subclass)]
pub struct CInitInfo {}

#[pymethods]
impl CInitInfo {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDeclarationsRecord::new(cd, ixval)).add_subclass(CInitInfo {})
    }

    #[getter]
    fn is_single(&self) -> bool {
        false
    }

    #[getter]
    fn is_compound(&self) -> bool {
        false
    }
}

/// Initializer of a simple variable.
///
/// - args[0]: index of initialization expression in cdictionary
#[derive(Clone)]
#[pyclass(extends = CInitInfo, frozen, subclass)]
pub struct CSingleInitInfo {}

#[pymethods]
impl CSingleInitInfo {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CInitInfo::new(cd, ixval)).add_subclass(CSingleInitInfo {})
    }
}

/// Initializer of a struct or array.
///
/// - args[0]: index of type of initializer in cdictionary
#[derive(Clone)]
#[pyclass(extends = CInitInfo, frozen, subclass)]
pub struct CCompoundInitInfo {}

// Unvalidated
#[pymethods]
impl CCompoundInitInfo {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CInitInfo::new(cd, ixval)).add_subclass(CCompoundInitInfo {})
    }
}

/// Component of a compound initializer.
///
/// - args[0]: index of offset expression in cdictionary
/// - args[1]: index of initinfo in cdeclarations
#[derive(Clone)]
#[pyclass(extends = CDeclarationsRecord, frozen, subclass)]
pub struct COffsetInitInfo {}

// Not validated?
#[pymethods]
impl COffsetInitInfo {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDeclarationsRecord::new(cd, ixval))
            .add_subclass(COffsetInitInfo {})
    }
}
