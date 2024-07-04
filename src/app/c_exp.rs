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

use pyo3::{intern, prelude::*};

use crate::{
    app::{
        c_const::CConst,
        c_dictionary::CDictionary,
        c_dictionary_record::{CDictionaryRecord, CDictionaryRegistryEntry},
    },
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_exp")?;
    module.add_class::<CExp>()?;
    module.add_class::<CExpConst>()?;
    Ok(module)
}

/// Base class for all expressions.
#[derive(Clone)]
#[pyclass(extends = CDictionaryRecord, frozen, subclass)]
pub struct CExp {}

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

    fn to_idict(slf: &Bound<Self>) -> BTreeMap<String, Py<PyAny>> {
        let c_dict_record = slf.borrow().into_super().into_super();
        BTreeMap::from([
            ("t".to_string(), c_dict_record.tags().to_object(slf.py())),
            ("a".to_string(), c_dict_record.args().to_object(slf.py())),
        ])
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>) -> String {
        format!("baseexp:{}", slf.into_super().into_super().tags()[0])
    }
}

/// Constant expression
///
/// - args[0]: constant
#[derive(Clone)]
#[pyclass(extends = CExp, frozen, subclass)]
struct CExpConst {}

#[pymethods]
impl CExpConst {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CExp::new(cd, ixval)).add_subclass(CExpConst {})
    }

    #[getter]
    fn is_constant(&self) -> bool {
        true
    }

    #[getter]
    fn constant<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, CConst>> {
        let py = slf.py();
        let c_dict_record = slf.borrow().into_super().into_super();
        let cd = c_dict_record.cd();
        let arg_0 = c_dict_record.into_super().args()[0];
        Ok(cd
            .call_method1(py, intern!(py, "get_constant"), (arg_0,))?
            .downcast_bound(py)?
            .clone())
    }

    fn get_strings(slf: &Bound<Self>) -> PyResult<Vec<String>> {
        // Use python runtime to resolve inheritance on get_strings
        Ok(CExpConst::constant(slf)?
            .call_method0(intern!(slf.py(), "get_strings"))?
            .extract()?)
    }

    fn to_dict(slf: &Bound<Self>) -> PyResult<BTreeMap<String, Py<PyAny>>> {
        Ok(BTreeMap::from([
            ("base".to_string(), "value".to_object(slf.py())),
            (
                "value".to_string(),
                CExpConst::constant(slf)?.into_any().unbind(),
            ),
        ]))
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        Ok(CExpConst::constant(slf)?.str()?.extract()?)
    }
}

inventory::submit! { CDictionaryRegistryEntry::new::<CExp, CExpConst>("const") }
