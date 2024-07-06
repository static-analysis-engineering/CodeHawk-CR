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
use std::collections::HashMap;

use pyo3::{exceptions::PyException, prelude::*};

use crate::cmdline::kendra::test_c_function_ref::TestCFunctionRef;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "test_spo_ref")?;
    module.add_class::<TestSPORef>()?;
    Ok(module)
}

#[pyclass(subclass)]
pub struct TestSPORef {
    #[pyo3(get)]
    testcfunctionref: Py<TestCFunctionRef>,
    #[pyo3(get)]
    refd: HashMap<String, Py<PyAny>>, // Supposed to be String, String
}

#[pymethods]
impl TestSPORef {
    #[new]
    pub fn new(
        testcfunctionref: Py<TestCFunctionRef>,
        refd: HashMap<String, Py<PyAny>>,
    ) -> TestSPORef {
        TestSPORef {
            testcfunctionref,
            refd,
        }
    }

    #[getter]
    pub fn line(&self, py: Python) -> PyResult<isize> {
        self.refd
            .get("line")
            .ok_or_else(|| PyException::new_err("'line' missing"))?
            .extract(py)
    }

    #[getter]
    fn context(&self, py: Python) -> PyResult<String> {
        self.refd
            .get("cfgctxt")
            .ok_or_else(|| PyException::new_err("'cfgctxt' missing"))?
            .extract(py)
    }

    #[getter]
    fn tgt_status(&self, py: Python) -> PyResult<String> {
        self.refd
            .get("tgtstatus")
            .ok_or_else(|| PyException::new_err("'tgtstatus' missing"))?
            .extract(py)
    }

    #[getter]
    fn status(&self, py: Python) -> PyResult<String> {
        self.refd
            .get("status")
            .ok_or_else(|| PyException::new_err("'status' missing"))?
            .extract(py)
    }

    #[getter]
    fn predicate(&self, py: Python) -> PyResult<String> {
        self.refd
            .get("predicate")
            .ok_or_else(|| PyException::new_err("'predicate' missing"))?
            .extract(py)
    }

    // Raw identifier because of PyO3 #4225
    #[getter]
    fn r#type(&self, py: Python) -> PyResult<String> {
        self.refd
            .get("type")
            .ok_or_else(|| PyException::new_err("'type' missing"))?
            .extract(py)
    }

    #[getter]
    fn argnr(&self, py: Python) -> PyResult<String> {
        self.refd
            .get("argnr")
            .ok_or_else(|| PyException::new_err("'argnr' missing"))?
            .extract(py)
    }

    #[getter]
    fn id(&self, py: Python) -> PyResult<(String, String)> {
        if self.r#type(py)? == "callsite" {
            Ok((self.predicate(py)?, self.argnr(py)?))
        } else {
            Ok(("?".to_string(), "?".to_string()))
        }
    }
}
