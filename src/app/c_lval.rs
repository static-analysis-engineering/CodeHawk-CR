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
pub struct CLval {
    cd: Py<CDictionary>,
    lhost_index: isize,
    offset_index: isize,
}

#[pymethods]
impl CLval {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        let lval = CLval {
            cd: cd.clone(),
            lhost_index: ixval.args()[0],
            offset_index: ixval.args()[1],
        };
        PyClassInitializer::from(CDictionaryRecord::new(cd, ixval)).add_subclass(lval)
    }

    #[getter]
    fn lhost<'a, 'b>(&'a self, py: Python<'b>) -> PyResult<Bound<'b, CLHost>> {
        Ok(self
            .cd
            .call_method1(py, intern!(py, "get_lhost"), (self.lhost_index,))?
            .downcast_bound::<CLHost>(py)?
            .clone())
    }

    #[getter]
    fn offset<'a, 'b>(&'a self, py: Python<'b>) -> PyResult<Bound<'b, COffset>> {
        Ok(self
            .cd
            .call_method1(py, intern!(py, "get_offset"), (self.offset_index,))?
            .downcast_bound::<COffset>(py)?
            .clone())
    }

    // Unvalidated
    fn has_variable(&self, py: Python, vid: isize) -> PyResult<bool> {
        // Method is overridden
        Ok(self
            .lhost(py)?
            .call_method1(intern!(py, "has_variable"), (vid,))?
            .extract()?)
    }

    // Unvalidated
    fn get_strings(&self, py: Python, vid: isize) -> PyResult<Vec<String>> {
        // Method is overridden
        let mut result: Vec<String> = self
            .lhost(py)?
            .call_method1(intern!(py, "get_strings"), (vid,))?
            .extract()?;
        // Method is overridden
        result.append(
            &mut self
                .offset(py)?
                .call_method1(intern!(py, "get_strings"), (vid,))?
                .extract()?,
        );
        return Ok(result);
    }

    // Unvalidated
    fn get_variable_uses(&self, py: Python, vid: isize) -> PyResult<Vec<String>> {
        // Method is overridden
        let mut result: Vec<String> = self
            .lhost(py)?
            .call_method1(intern!(py, "get_variable_uses"), (vid,))?
            .extract()?;
        // Method is overridden
        result.append(
            &mut self
                .offset(py)?
                .call_method1(intern!(py, "get_variable_uses"), (vid,))?
                .extract()?,
        );
        return Ok(result);
    }

    // Unvalidated
    fn has_variable_deref(&self, py: Python, vid: isize) -> PyResult<bool> {
        // Method is overridden
        Ok(self
            .lhost(py)?
            .call_method1(intern!(py, "has_variable_deref"), (vid,))?
            .extract()?)
    }

    // Unvalidated
    fn has_ref_type(&self, py: Python) -> PyResult<bool> {
        // Method is overridden
        Ok(self
            .lhost(py)?
            .call_method0(intern!(py, "has_ref_type"))?
            .extract()?)
    }

    // Unvalidated
    fn to_dict(&self, py: Python) -> PyResult<BTreeMap<&'static str, Py<PyAny>>> {
        Ok(BTreeMap::from([
            // Method is overridden
            (
                "lhost",
                self.lhost(py)?
                    .call_method0(intern!(py, "to_dict"))?
                    .unbind(),
            ),
            // Method is overridden
            (
                "lhost",
                self.offset(py)?
                    .call_method0(intern!(py, "to_dict"))?
                    .unbind(),
            ),
        ]))
    }

    #[pyo3(name = "__str__")]
    fn str(&self, py: Python) -> PyResult<String> {
        Ok(self.lhost(py)?.str()?.extract::<String>()?
            + &self.offset(py)?.str()?.extract::<String>()?)
    }
}
