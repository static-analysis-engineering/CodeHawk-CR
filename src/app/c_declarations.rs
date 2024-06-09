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
use std::collections::{BTreeMap, BTreeSet};

use pyo3::prelude::*;

use crate::app::{
    c_comp_info::CCompInfo,
    c_dictionary::CDictionary,
    c_field_info::CFieldInfo,
    c_file::CFile,
    c_init_info::{CInitInfo, COffsetInitInfo},
    c_location::CLocation,
    c_typ::CTyp,
};

pyo3::import_exception!(chc.util.fileutil, CHError);

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_declarations")?;
    module.add_class::<CDeclarations>()?;
    Ok(module)
}

/// Abstract super class for CGlobalDeclarations and CFileDeclarations.
#[derive(Clone)]
#[pyclass(subclass)]
pub struct CDeclarations {}

#[pymethods]
impl CDeclarations {
    #[new]
    pub fn new() -> CDeclarations {
        CDeclarations {}
    }

    #[getter]
    fn dictionary(&self) -> PyResult<Py<CDictionary>> {
        Err(CHError::new_err("unimplemented"))
    }

    #[getter]
    fn cfile(&self) -> PyResult<Py<CFile>> {
        Err(CHError::new_err("unimplemented"))
    }

    fn get_initinfo(&self, _ix: isize) -> PyResult<Py<CInitInfo>> {
        Err(CHError::new_err("unimplemented"))
    }

    fn get_fieldinfo(&self, _ix: isize) -> PyResult<Py<CFieldInfo>> {
        Err(CHError::new_err("unimplemented"))
    }

    fn get_offset_init(&self, _ix: isize) -> PyResult<Py<COffsetInitInfo>> {
        Err(CHError::new_err("unimplemented"))
    }

    fn get_compinfo_by_ckey(&self, _ix: isize) -> PyResult<Py<CCompInfo>> {
        Err(CHError::new_err("unimplemented"))
    }

    fn get_location(&self, _ix: isize) -> PyResult<Py<CLocation>> {
        Err(CHError::new_err(
            "Global declarations does not keep a location.",
        ))
    }

    #[getter]
    fn varinfo_storage_classes(&self) -> PyResult<BTreeMap<isize, BTreeSet<String>>> {
        Err(CHError::new_err(
            "File declarations does not keep storage classes.",
        ))
    }

    fn expand(&self) -> PyResult<Py<CTyp>> {
        Err(CHError::new_err(
            "Types should be expanded at the file level.",
        ))
    }
}
