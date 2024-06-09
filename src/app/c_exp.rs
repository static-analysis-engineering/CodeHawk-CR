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
    let module = PyModule::new_bound(py, "c_exp")?;
    module.add_class::<CExp>()?;
    Ok(module)
}

/// Base class for all expressions.
#[derive(Clone)]
#[pyclass(extends = CDictionaryRecord, frozen, subclass)]
struct CExp {}

#[pymethods]
impl CExp {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDictionaryRecord::new(cd, ixval)).add_subclass(CExp {})
    }

    #[getter]
    fn is_binop(&self) -> bool {
        false
    }

    #[getter]
    fn is_caste(&self) -> bool {
        false
    }

    #[getter]
    fn is_constant(&self) -> bool {
        false
    }

    #[getter]
    fn is_lval(&self) -> bool {
        false
    }

    #[getter]
    fn is_question(&self) -> bool {
        false
    }

    #[getter]
    fn is_sizeof(&self) -> bool {
        false
    }

    #[getter]
    fn is_sizeofe(&self) -> bool {
        false
    }

    #[getter]
    fn is_sizeofstr(&self) -> bool {
        false
    }

    #[getter]
    fn is_addrof(&self) -> bool {
        false
    }

    #[getter]
    fn is_startof(&self) -> bool {
        false
    }

    #[getter]
    fn is_unop(&self) -> bool {
        false
    }

    #[getter]
    fn is_alignof(&self) -> bool {
        false
    }

    #[getter]
    fn is_alignofe(&self) -> bool {
        false
    }

    #[getter]
    fn is_fn_app(&self) -> bool {
        false
    }

    #[getter]
    fn is_cn_app(&self) -> bool {
        false
    }

    fn has_variable(&self, _vid: isize) -> bool {
        false
    }

    fn has_variable_op(&self, _vid: isize, _op: &str) -> bool {
        false
    }

    fn get_strings(&self) -> Vec<String> {
        vec![]
    }

    fn get_variable_uses(&self, _vid: isize) -> isize {
        0
    }

    fn to_dict(&self) -> BTreeMap<String, String> {
        BTreeMap::from([("base".to_string(), "exp".to_string())])
    }

    fn to_idict(slf: PyRef<Self>, py: Python) -> BTreeMap<String, Py<PyAny>> {
        let c_dict_record = slf.into_super().into_super();
        BTreeMap::from([
            ("t".to_string(), c_dict_record.tags().to_vec().into_py(py)),
            ("a".to_string(), c_dict_record.args().to_vec().into_py(py)),
        ])
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>) -> String {
        format!("baseexp:{}", slf.into_super().into_super().tags()[0])
    }
}