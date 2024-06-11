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
use std::collections::BTreeMap;

use pyo3::prelude::*;

use crate::{
    app::{c_dictionary::CDictionary, c_dictionary_record::CDictionaryRecord},
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_offset")?;
    module.add_class::<COffset>()?;
    module.add_class::<CNoOffset>()?;
    Ok(module)
}

/// Base class for an expression offset.
#[derive(Clone)]
#[pyclass(extends = CDictionaryRecord, frozen, subclass)]
struct COffset {}

#[pymethods]
impl COffset {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDictionaryRecord::new(cd, ixval)).add_subclass(COffset {})
    }

    fn has_offset(&self) -> bool {
        true
    }

    #[getter]
    fn is_no_offset(&self) -> bool {
        false
    }

    #[getter]
    fn is_field(&self) -> bool {
        false
    }

    #[getter]
    fn is_index(&self) -> bool {
        false
    }

    fn get_strings(&self) -> Vec<String> {
        vec![]
    }

    fn get_variable_uses(&self, _vid: isize) -> isize {
        0
    }

    fn to_dict(&self) -> BTreeMap<String, String> {
        BTreeMap::from([("base".to_string(), "offset".to_string())])
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>) -> String {
        format!("offsetbase: {}", slf.into_super().into_super().tags()[0])
    }
}

#[derive(Clone)]
#[pyclass(extends = COffset, frozen, subclass)]
struct CNoOffset {}

#[pymethods]
impl CNoOffset {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(COffset::new(cd, ixval)).add_subclass(CNoOffset {})
    }

    fn has_offset(&self) -> bool {
        false
    }

    #[getter]
    fn is_no_offset(&self) -> bool {
        true
    }

    fn to_dict(&self) -> BTreeMap<String, String> {
        BTreeMap::from([("base".to_string(), "no-offset".to_string())])
    }

    #[pyo3(name = "__str__")]
    fn str(&self) -> String {
        "".to_string()
    }
}
