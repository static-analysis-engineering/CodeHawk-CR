/*
------------------------------------------------------------------------------
CodeHawk C Analyzer
Author: Henny Sipma
------------------------------------------------------------------------------
The MIT License (MIT)

Copyright (c) 2017-2020 Kestrel Technology LLC
Copyright (c) 2021-2022 Henny B. Sipma
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
    app::{c_file::CFile, c_file_declarations::CFileDeclarations},
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "assign_dictionary_record")?;
    module.add_class::<AssignDictionaryRecord>()?;
    Ok(module)
}

pyo3::import_exception!(chcc.util.fileutil, CHCError);

// Unvalidated
/// Base class for all objects kept in the CFileAssignmentDictionary.
#[pyclass(extends = IndexedTableValue, frozen, subclass)]
pub struct AssignDictionaryRecord {
    #[pyo3(get)]
    ad: Py<PyAny>, // CFileAssignmentDictionary
}

// Unvalidated
#[pymethods]
impl AssignDictionaryRecord {
    #[new]
    pub fn new(
        ad: Py<PyAny>,
        ixval: IndexedTableValue,
    ) -> (AssignDictionaryRecord, IndexedTableValue) {
        (AssignDictionaryRecord { ad }, ixval)
    }

    #[getter]
    fn cfile<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, CFile>> {
        Ok(slf
            .get()
            .ad
            .bind(slf.py())
            .getattr(intern!(slf.py(), "cfile"))?
            .downcast()?
            .clone())
    }

    #[getter]
    fn cd<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, PyAny>> {
        // CFileDictionary
        AssignDictionaryRecord::cfile(slf)?.getattr(intern!(slf.py(), "dictionary"))
    }

    #[getter]
    fn cdecls<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, CFileDeclarations>> {
        Ok(AssignDictionaryRecord::cfile(slf)?
            .getattr(intern!(slf.py(), "dictionary"))?
            .downcast()?
            .clone())
    }
}
