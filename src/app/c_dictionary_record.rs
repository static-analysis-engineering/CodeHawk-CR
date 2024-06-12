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
    app::{c_declarations::CDeclarations, c_dictionary::CDictionary},
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_dictionary_record")?;
    module.add_class::<CDictionaryRecord>()?;
    module.add_class::<CDeclarationsRecord>()?;
    Ok(module)
}

/// Base class for all objects kept in the CDictionary
#[derive(Clone)]
#[pyclass(extends = IndexedTableValue, frozen, subclass)]
pub struct CDictionaryRecord {
    #[pyo3(get)]
    cd: Py<CDictionary>,
}

#[pymethods]
impl CDictionaryRecord {
    #[new]
    pub fn new(
        cd: Py<CDictionary>,
        ixval: IndexedTableValue,
    ) -> (CDictionaryRecord, IndexedTableValue) {
        (CDictionaryRecord { cd }, ixval)
    }

    #[getter]
    pub fn decls(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.cd.getattr(py, intern!(py, "decls"))
    }
}

impl CDictionaryRecord {
    pub fn cd(&self) -> Py<CDictionary> {
        self.cd.clone()
    }
}

/// Base class for all objects kept in the CFileDeclarations.
#[derive(Clone)]
#[pyclass(extends = IndexedTableValue, frozen, subclass)]
pub struct CDeclarationsRecord {
    #[pyo3(get)]
    decls: Py<CDeclarations>,
}

#[pymethods]
impl CDeclarationsRecord {
    #[new]
    pub fn new(
        decls: Py<CDeclarations>,
        ixval: IndexedTableValue,
    ) -> (CDeclarationsRecord, IndexedTableValue) {
        (CDeclarationsRecord { decls }, ixval)
    }

    #[getter]
    pub fn dictionary(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.decls.getattr(py, intern!(py, "dictionary"))
    }
}

impl CDeclarationsRecord {
    pub fn decls(&self) -> Py<CDeclarations> {
        self.decls.clone()
    }
}
