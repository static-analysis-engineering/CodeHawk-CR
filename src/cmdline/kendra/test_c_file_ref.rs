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
use std::collections::{BTreeMap, HashMap};

use once_cell::sync::OnceCell;
use pyo3::prelude::*;

use crate::cmdline::kendra::{test_c_function_ref::TestCFunctionRef, test_set_ref::TestSetRef};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "test_c_file_ref")?;
    module.add_class::<TestCFileRef>()?;
    Ok(module)
}

#[pyclass(frozen, subclass)]
pub struct TestCFileRef {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    testsetref: Py<TestSetRef>,
    #[pyo3(get)]
    refd: HashMap<String, Py<PyAny>>,
    functions: OnceCell<BTreeMap<String, Py<TestCFunctionRef>>>,
}

#[pymethods]
impl TestCFileRef {
    #[new]
    pub fn new(
        testsetref: Py<TestSetRef>,
        name: String,
        refd: HashMap<String, Py<PyAny>>,
    ) -> TestCFileRef {
        TestCFileRef {
            testsetref,
            name,
            refd,
            functions: OnceCell::new(),
        }
    }

    #[getter]
    fn functions(
        py_self: Py<Self>,
        py: Python,
    ) -> PyResult<BTreeMap<String, Py<TestCFunctionRef>>> {
        let slf = py_self.get();
        slf.functions
            .get_or_try_init(|| {
                let mut functions = BTreeMap::new();
                let Some(dict) = slf.refd.get("functions") else {
                    return Ok(functions);
                };
                let fn_map: BTreeMap<String, BTreeMap<String, Py<PyAny>>> = dict.extract(py)?;
                for (f, fdata) in fn_map {
                    functions.insert(
                        f.clone(),
                        Py::new(py, TestCFunctionRef::new(py_self.clone(), f, fdata))?,
                    );
                }
                Ok(functions)
            })
            .cloned()
    }

    #[getter]
    fn functionnames(py_self: Py<Self>, py: Python) -> PyResult<Vec<String>> {
        Ok(TestCFileRef::functions(py_self, py)?
            .keys()
            .cloned()
            .collect()) // Sorting comes from collection
    }

    // Seems unused
    fn get_function(
        py_self: Py<Self>,
        py: Python,
        fname: &str,
    ) -> PyResult<Option<Py<TestCFunctionRef>>> {
        Ok(TestCFileRef::functions(py_self, py)?.get(fname).cloned())
    }

    fn has_domains(&self, py: Python) -> PyResult<bool> {
        Ok(!self.domains(py)?.is_empty())
    }

    #[getter]
    fn domains(&self, py: Python) -> PyResult<Vec<String>> {
        let Some(domains) = self.refd.get("domains") else {
            return Ok(Vec::new());
        };
        domains.extract(py)
    }

    // Seems unused
    fn has_spos(py_self: Py<Self>, py: Python) -> PyResult<bool> {
        for f in TestCFileRef::functions(py_self, py)?.into_values() {
            if TestCFunctionRef::has_spos(f.bind(py))? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
