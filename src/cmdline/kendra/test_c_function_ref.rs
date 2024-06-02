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

use pyo3::prelude::*;

use crate::cmdline::kendra::test_c_file_ref::TestCFileRef;
use crate::cmdline::kendra::test_ppo_ref::TestPPORef;
use crate::cmdline::kendra::test_spo_ref::TestSPORef;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "test_c_function_ref")?;
    module.add_class::<TestCFunctionRef>()?;
    Ok(module)
}

#[derive(Clone)]
#[pyclass(subclass)]
pub struct TestCFunctionRef {
    #[pyo3(get)]
    testcfileref: Py<TestCFileRef>,
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    refd: HashMap<String, Py<PyAny>>, // Supposed to be HashMap<String, String>
    // TODO: use OnceLock when once_cell_try stabilizes
    line_ppos: HashMap<isize, Vec<Py<TestPPORef>>>,
    line_spos: HashMap<isize, Vec<Py<TestSPORef>>>,
}

#[pymethods]
impl TestCFunctionRef {
    #[new]
    fn new(
        testcfileref: Py<TestCFileRef>,
        name: String,
        refd: HashMap<String, Py<PyAny>>,
    ) -> TestCFunctionRef {
        TestCFunctionRef {
            testcfileref,
            name,
            refd,
            line_ppos: HashMap::new(),
            line_spos: HashMap::new(),
        }
    }

    #[getter]
    fn line_ppos(slf: Py<Self>, py: Python) -> PyResult<HashMap<isize, Vec<Py<TestPPORef>>>> {
        let mut slf_mut_ref = slf.borrow_mut(py);
        if slf_mut_ref.line_ppos.is_empty() {
            if let Some(ppos) = slf_mut_ref.refd.get("ppos") {
                for p in ppos.extract::<Vec<Py<PyAny>>>(py)? {
                    let ppo = Py::new(py, TestPPORef::new(slf.clone(), p.extract(py)?))?;
                    let line = ppo.borrow(py).line(py)?;
                    slf_mut_ref
                        .line_ppos
                        .entry(line)
                        .or_insert(Vec::new())
                        .push(ppo)
                }
            }
        }
        Ok(slf_mut_ref.line_ppos.clone())
    }

    #[getter]
    fn line_spos(slf: Py<Self>, py: Python) -> PyResult<HashMap<isize, Vec<Py<TestSPORef>>>> {
        let mut slf_mut_ref = slf.borrow_mut(py);
        if slf_mut_ref.line_spos.is_empty() {
            if let Some(spos) = slf_mut_ref.refd.get("spos") {
                for p in spos.extract::<Vec<Py<PyAny>>>(py)? {
                    let spo = Py::new(py, TestSPORef::new(slf.clone(), p.extract(py)?))?;
                    let line = spo.borrow(py).line(py)?;
                    slf_mut_ref
                        .line_spos
                        .entry(line)
                        .or_insert(Vec::new())
                        .push(spo)
                }
            }
        }
        Ok(slf_mut_ref.line_spos.clone())
    }
}
