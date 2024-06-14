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
//! Left-hand side value.

use std::collections::BTreeMap;

use pyo3::{intern, prelude::*};

use crate::{
    app::{
        c_dictionary::CDictionary, c_dictionary_record::CDictionaryRecord, c_lhost::CLHost,
        c_offset::COffset,
    },
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_lval")?;
    module.add_class::<CLval>()?;
    Ok(module)
}

/// Left-hand side value.
///
/// * args[0]: index of lhost in cdictionary
/// * args[1]: index of offset in cdictionary
#[derive(Clone)]
#[pyclass(extends = CDictionaryRecord, frozen)]
pub struct CLval {}

#[pymethods]
impl CLval {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDictionaryRecord::new(cd, ixval)).add_subclass(CLval {})
    }

    #[getter]
    fn lhost<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, CLHost>> {
        let c_dict_entry = slf.borrow().into_super();
        let cd = c_dict_entry.cd();
        let arg_0 = c_dict_entry.into_super().args()[0];
        Ok(cd
            .call_method1(slf.py(), intern!(slf.py(), "get_lhost"), (arg_0,))?
            .downcast_bound::<CLHost>(slf.py())?
            .clone())
    }

    #[getter]
    fn offset<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, COffset>> {
        let c_dict_entry = slf.borrow().into_super();
        let cd = c_dict_entry.cd();
        let arg_1 = c_dict_entry.into_super().args()[1];
        Ok(cd
            .call_method1(slf.py(), intern!(slf.py(), "get_offset"), (arg_1,))?
            .downcast_bound::<COffset>(slf.py())?
            .clone())
    }

    // Unvalidated
    fn has_variable(slf: &Bound<Self>, vid: isize) -> PyResult<bool> {
        // Method is overridden
        Ok(CLval::lhost(slf)?
            .call_method1(intern!(slf.py(), "has_variable"), (vid,))?
            .extract()?)
    }

    // Unvalidated
    fn get_strings(slf: &Bound<Self>, vid: isize) -> PyResult<Vec<String>> {
        // Method is overridden
        let mut result: Vec<String> = CLval::lhost(slf)?
            .call_method1(intern!(slf.py(), "get_strings"), (vid,))?
            .extract()?;
        // Method is overridden
        result.append(
            &mut CLval::offset(slf)?
                .call_method1(intern!(slf.py(), "get_strings"), (vid,))?
                .extract()?,
        );
        return Ok(result);
    }

    // Unvalidated
    fn get_variable_uses(slf: &Bound<Self>, vid: isize) -> PyResult<Vec<String>> {
        // Method is overridden
        let mut result: Vec<String> = CLval::lhost(slf)?
            .call_method1(intern!(slf.py(), "get_variable_uses"), (vid,))?
            .extract()?;
        // Method is overridden
        result.append(
            &mut CLval::offset(slf)?
                .call_method1(intern!(slf.py(), "get_variable_uses"), (vid,))?
                .extract()?,
        );
        return Ok(result);
    }

    // Unvalidated
    fn has_variable_deref(slf: &Bound<Self>, vid: isize) -> PyResult<bool> {
        // Method is overridden
        Ok(CLval::lhost(slf)?
            .call_method1(intern!(slf.py(), "has_variable_deref"), (vid,))?
            .extract()?)
    }

    // Unvalidated
    fn has_ref_type(slf: &Bound<Self>) -> PyResult<bool> {
        // Method is overridden
        Ok(CLval::lhost(slf)?
            .call_method0(intern!(slf.py(), "has_ref_type"))?
            .extract()?)
    }

    // Unvalidated
    fn to_dict(slf: &Bound<Self>) -> PyResult<BTreeMap<String, Py<PyAny>>> {
        Ok(BTreeMap::from([
            // Method is overridden
            (
                "lhost".to_string(),
                CLval::lhost(slf)?
                    .call_method0(intern!(slf.py(), "to_dict"))?
                    .unbind(),
            ),
            // Method is overridden
            (
                "lhost".to_string(),
                CLval::offset(slf)?
                    .call_method0(intern!(slf.py(), "to_dict"))?
                    .unbind(),
            ),
        ]))
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        Ok(CLval::lhost(slf)?.str()?.extract::<String>()?
            + &CLval::offset(slf)?.str()?.extract::<String>()?)
    }
}
