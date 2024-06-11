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
    app::{c_dictionary::CDictionary, c_dictionary_record::CDictionaryRecord},
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_offset")?;
    module.add_class::<CFieldOffset>()?;
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

/// Field offset
///
/// * tags[1]: fieldname
///
/// * args[0]: ckey of the containing struct
/// * args[1]: index of sub-offset in cdictionary
#[derive(Clone)]
#[pyclass(extends = COffset, frozen, subclass)]
struct CFieldOffset {}

#[pymethods]
impl CFieldOffset {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(COffset::new(cd, ixval)).add_subclass(CFieldOffset {})
    }

    #[getter]
    fn fieldname(slf: PyRef<Self>) -> String {
        slf.into_super().into_super().into_super().tags()[1].clone()
    }

    #[getter]
    fn ckey(slf: PyRef<Self>) -> isize {
        slf.into_super().into_super().into_super().args()[0]
    }

    #[getter]
    fn offset<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, COffset>> {
        let py = slf.py();
        let c_dict_record = slf.borrow().into_super().into_super();
        let cd = c_dict_record.cd();
        let arg_1 = c_dict_record.into_super().args()[1];
        Ok(cd
            .call_method1(py, intern!(py, "get_offset"), (arg_1,))?
            .downcast_bound(py)?
            .clone())
    }

    #[getter]
    fn is_field(&self) -> bool {
        true
    }

    // Seems unused
    fn to_dict(slf: &Bound<Self>) -> PyResult<BTreeMap<String, Py<PyAny>>> {
        let py = slf.py();
        let mut map = BTreeMap::from([
            ("base".to_string(), "field-offset".to_object(py)),
            (
                "field".to_string(),
                CFieldOffset::fieldname(slf.borrow()).to_object(py),
            ),
        ]);
        let offset = CFieldOffset::offset(slf)?;
        // Resolve call with python interpreter for possible override
        if offset
            .call_method0(intern!(slf.py(), "has_offset"))?
            .extract()?
        {
            let inner = offset.call_method0(intern!(slf.py(), "to_dict"))?;
            map.insert("offset".to_string(), inner.unbind());
        }
        Ok(map)
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        // Resolve call with python interpret for possible override
        let offset = if slf
            .call_method0(intern!(slf.py(), "has_offset"))?
            .extract()?
        {
            CFieldOffset::offset(slf)?.str()?.extract()?
        } else {
            "".to_string()
        };
        Ok(format!(
            ".{}{offset}",
            CFieldOffset::fieldname(slf.borrow())
        ))
    }
}
