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
    let module = PyModule::new_bound(py, "test_ppo_ref")?;
    module.add_class::<TestPPORef>()?;
    Ok(module)
}

#[derive(Clone)]
#[pyclass(subclass)]
pub struct TestPPORef {
    #[pyo3(get)]
    testcfunctionref: Py<TestCFunctionRef>,
    #[pyo3(get)]
    refd: HashMap<String, Py<PyAny>>, // Suppoed to be String, String?
}

#[pymethods]
impl TestPPORef {
    #[new]
    pub fn new(
        testcfunctionref: Py<TestCFunctionRef>,
        refd: HashMap<String, Py<PyAny>>,
    ) -> TestPPORef {
        TestPPORef {
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
    fn cfg_context(&self, py: Python) -> PyResult<String> {
        self.refd
            .get("cfgctxt")
            .ok_or_else(|| PyException::new_err("'cfgctxt' missing"))?
            .extract(py)
    }

    #[getter]
    fn exp_context(&self, py: Python) -> PyResult<String> {
        self.refd
            .get("expctxt")
            .ok_or_else(|| PyException::new_err("'expctxt' missing"))?
            .extract(py)
    }

    #[getter]
    fn context(&self, py: Python) -> PyResult<(String, String)> {
        Ok((self.cfg_context(py)?, self.exp_context(py)?))
    }

    #[getter]
    fn context_string(&self, py: Python) -> PyResult<String> {
        Ok(format!(
            "({},{})",
            self.cfg_context(py)?,
            self.exp_context(py)?
        ))
    }

    #[getter]
    fn predicate(&self, py: Python) -> PyResult<String> {
        self.refd
            .get("predicate")
            .ok_or_else(|| PyException::new_err("'predicate' missing"))?
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
}
