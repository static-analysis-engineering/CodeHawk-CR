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
use itertools::Itertools;
use pyo3::{intern, prelude::*};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "api_assumption")?;
    module.add_class::<ApiAssumption>()?;
    Ok(module)
}

/// Assumption on the function api.
///
/// Args:
///     capi (CFunctionApi): parent function api
///     id (int): identification number
///     predicate (CPOPredicate): expression of the assumption
///     ppos (List[int]): list of primary proof obligation id's that depend on
///        this assumption
///     spos (List[int]): list of supporting proof obligation id's that depend
///        on this assumption
///     isglobal (bool=False): assumption holds globally
///     isfile (bool=False): assumption holds for the entire c-file
#[derive(Clone)]
#[pyclass(frozen)]
pub struct ApiAssumption {
    #[pyo3(get)]
    capi: Py<PyAny>,
    #[pyo3(get)]
    cfun: Py<PyAny>,
    #[pyo3(get)]
    id: isize,
    #[pyo3(get)]
    predicate: Py<PyAny>,
    #[pyo3(get)]
    ppos: Vec<isize>,
    #[pyo3(get)]
    spos: Vec<isize>,
    #[pyo3(get)]
    isglobal: bool,
    #[pyo3(get)]
    isfile: bool,
}

#[pymethods]
impl ApiAssumption {
    #[new]
    fn new(
        py: Python,
        capi: Py<PyAny>,
        id: isize,
        predicate: Py<PyAny>,
        ppos: Vec<isize>,
        spos: Vec<isize>,
        isglobal: bool,
        isfile: bool,
    ) -> PyResult<ApiAssumption> {
        let cfun = capi.getattr(py, intern!(py, "cfun"))?;
        Ok(ApiAssumption {
            capi,
            cfun,
            id,
            predicate,
            ppos,
            spos,
            isglobal,
            isfile,
        })
    }

    #[pyo3(name = "__str__")]
    fn str(&self) -> String {
        let strppos = if self.ppos.is_empty() {
            "".to_string()
        } else {
            format!(
                "\n      --Dependent ppo's: [{}]",
                self.ppos.iter().join(",")
            )
        };
        let strspos = if self.spos.is_empty() {
            "".to_string()
        } else {
            format!(
                "\n      --Dependent spo's: [{}]",
                self.spos.iter().join(",")
            )
        };
        let isglobal = if self.isglobal { " (global)" } else { "" };
        format!(
            "{} {} {isglobal}{strppos}{strspos}",
            self.id, self.predicate
        )
    }
}
