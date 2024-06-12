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
    app::{c_dictionary::CDictionary, c_dictionary_record::CDictionaryRecord},
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_attributes")?;
    module.add_class::<CAttr>()?;
    module.add_class::<CAttrCons>()?;
    module.add_class::<CAttrInt>()?;
    module.add_class::<CAttrStr>()?;
    module.add_class::<CAttributes>()?;
    Ok(module)
}

/// Attribute that comes with a C type.
#[derive(Clone)]
#[pyclass(extends = CDictionaryRecord, frozen, subclass)]
struct CAttr {}

#[pymethods]
impl CAttr {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDictionaryRecord::new(cd, ixval)).add_subclass(CAttr {})
    }

    #[getter]
    fn is_int(&self) -> bool {
        false
    }

    #[getter]
    fn is_str(&self) -> bool {
        false
    }

    #[getter]
    fn is_cons(&self) -> bool {
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
    fn is_sizeofs(&self) -> bool {
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
    fn is_alignofs(&self) -> bool {
        false
    }

    #[getter]
    fn is_unop(&self) -> bool {
        false
    }

    #[getter]
    fn is_binop(&self) -> bool {
        false
    }

    #[getter]
    fn is_dot(&self) -> bool {
        false
    }

    #[getter]
    fn is_star(&self) -> bool {
        false
    }

    #[getter]
    fn is_addrof(&self) -> bool {
        false
    }

    #[getter]
    fn is_index(&self) -> bool {
        false
    }

    #[getter]
    fn is_question(&self) -> bool {
        false
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>) -> String {
        format!("attrparam:{}", slf.into_super().into_super().tags()[0])
    }
}

/// Integer attribute.
///
/// args[0]: integer value
#[derive(Clone)]
#[pyclass(extends = CAttr, frozen, subclass)]
struct CAttrInt {}

#[pymethods]
impl CAttrInt {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CAttr::new(cd, ixval)).add_subclass(CAttrInt {})
    }

    #[getter]
    fn intvalue(slf: PyRef<Self>) -> isize {
        slf.into_super().into_super().into_super().args()[0]
    }

    #[getter]
    fn is_int(&self) -> bool {
        true
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>) -> String {
        format!("aint({})", CAttrInt::intvalue(slf))
    }
}

/// String attribute.
///
/// * args[0]: index in string table of string attribute
#[derive(Clone)]
#[pyclass(extends = CAttr, frozen, subclass)]
struct CAttrStr {}

// Unvalidated
#[pymethods]
impl CAttrStr {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CAttr::new(cd, ixval)).add_subclass(CAttrStr {})
    }

    #[getter]
    fn stringvalue(slf: Py<Self>, py: Python) -> PyResult<String> {
        let args_0 = slf.borrow(py).into_super().into_super().into_super().args()[0];
        slf.borrow(py)
            .into_super()
            .into_super()
            .cd()
            .call_method1(py, intern!(py, "get_string"), (args_0,))?
            .extract(py)
    }

    #[getter]
    fn is_str(&self) -> bool {
        true
    }

    #[pyo3(name = "__str__")]
    fn str(slf: Py<Self>, py: Python) -> PyResult<String> {
        Ok(format!("astr({})", CAttrStr::stringvalue(slf, py)?))
    }
}

/// Constructed attributes.
///
/// * tags[1]: name
/// * args[0..]: indices of attribute parameters in cdictionary.
#[derive(Clone)]
#[pyclass(extends = CAttr, frozen, subclass)]
struct CAttrCons {}

#[pymethods]
impl CAttrCons {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CAttr::new(cd, ixval)).add_subclass(CAttrCons {})
    }

    #[getter]
    fn name(slf: PyRef<Self>) -> String {
        slf.into_super().into_super().into_super().tags()[1].clone()
    }

    #[getter]
    fn params(slf: Py<Self>, py: Python) -> PyResult<Vec<Bound<CAttr>>> {
        let cd = slf.borrow(py).into_super().into_super().cd();
        slf.borrow(py)
            .into_super()
            .into_super()
            .into_super()
            .args()
            .iter()
            .map(|i| cd.call_method1(py, intern!(py, "get_attrparam"), (*i,)))
            .map(|ins| Ok(ins?.downcast_bound::<CAttr>(py)?.clone()))
            .collect()
    }

    #[getter]
    fn is_cons(&self) -> bool {
        true
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>) -> String {
        format!("acons({})", CAttrCons::name(slf))
    }
}

#[derive(Clone)]
#[pyclass(extends = CDictionaryRecord, frozen, subclass)]
pub struct CAttributes {}

#[pymethods]
impl CAttributes {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDictionaryRecord::new(cd, ixval)).add_subclass(CAttributes {})
    }
}
