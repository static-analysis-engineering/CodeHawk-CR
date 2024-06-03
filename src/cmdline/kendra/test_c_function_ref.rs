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

use once_cell::sync::OnceCell;
use pyo3::prelude::*;

use crate::cmdline::kendra::{
    test_c_file_ref::TestCFileRef, test_ppo_ref::TestPPORef, test_spo_ref::TestSPORef,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "test_c_function_ref")?;
    module.add_class::<TestCFunctionRef>()?;
    Ok(module)
}

#[derive(Clone)]
#[pyclass(frozen, subclass)]
pub struct TestCFunctionRef {
    #[pyo3(get)]
    testcfileref: Py<TestCFileRef>,
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    refd: HashMap<String, Py<PyAny>>, // Supposed to be HashMap<String, String>
    // TODO: use OnceLock when once_cell_try stabilizes
    line_ppos: OnceCell<HashMap<isize, Vec<Py<TestPPORef>>>>,
    line_spos: OnceCell<HashMap<isize, Vec<Py<TestSPORef>>>>,
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
            line_ppos: OnceCell::new(),
            line_spos: OnceCell::new(),
        }
    }

    #[getter]
    fn line_ppos(py_self: Py<Self>, py: Python) -> PyResult<HashMap<isize, Vec<Py<TestPPORef>>>> {
        let slf = py_self.get();
        slf.line_ppos
            .get_or_try_init(|| {
                let mut line_ppos = HashMap::new();
                if let Some(ppos) = slf.refd.get("ppos") {
                    for p in ppos.extract::<Vec<Py<PyAny>>>(py)? {
                        let ppo = Py::new(py, TestPPORef::new(py_self.clone(), p.extract(py)?))?;
                        let line = ppo.borrow(py).line(py)?;
                        line_ppos.entry(line).or_insert(Vec::new()).push(ppo)
                    }
                }
                Ok(line_ppos)
            })
            .cloned()
    }

    #[getter]
    fn line_spos(py_self: Py<Self>, py: Python) -> PyResult<HashMap<isize, Vec<Py<TestSPORef>>>> {
        let slf = py_self.get();
        slf.line_spos
            .get_or_try_init(|| {
                let mut line_spos = HashMap::new();
                if let Some(spos) = slf.refd.get("spos") {
                    for p in spos.extract::<Vec<Py<PyAny>>>(py)? {
                        let spo = Py::new(py, TestSPORef::new(py_self.clone(), p.extract(py)?))?;
                        let line = spo.borrow(py).line(py)?;
                        line_spos.entry(line).or_insert(Vec::new()).push(spo)
                    }
                }
                Ok(line_spos)
            })
            .cloned()
    }
}
