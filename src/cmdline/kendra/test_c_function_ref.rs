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

use itertools::Itertools;
use once_cell::sync::OnceCell;
use pyo3::{exceptions::PyException, prelude::*, types::PyList};

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
    refd: BTreeMap<String, Py<PyAny>>, // Supposed to be HashMap<String, String>
    // TODO: use OnceLock when once_cell_try stabilizes
    line_ppos: OnceCell<BTreeMap<isize, Vec<Py<TestPPORef>>>>,
    line_spos: OnceCell<BTreeMap<isize, Vec<Py<TestSPORef>>>>,
}

#[pymethods]
impl TestCFunctionRef {
    #[new]
    pub fn new(
        testcfileref: Py<TestCFileRef>,
        name: String,
        refd: BTreeMap<String, Py<PyAny>>,
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
    fn line_ppos(py_self: Py<Self>, py: Python) -> PyResult<BTreeMap<isize, Vec<Py<TestPPORef>>>> {
        let slf = py_self.get();
        slf.line_ppos
            .get_or_try_init(|| {
                let mut line_ppos = BTreeMap::new();
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
    fn line_spos(py_self: Py<Self>, py: Python) -> PyResult<BTreeMap<isize, Vec<Py<TestSPORef>>>> {
        let slf = py_self.get();
        slf.line_spos
            .get_or_try_init(|| {
                let mut line_spos = BTreeMap::new();
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

    #[getter]
    fn ppos(py_self: Py<Self>, py: Python) -> PyResult<Vec<Py<TestPPORef>>> {
        Ok(TestCFunctionRef::line_ppos(py_self, py)?
            .values()
            .flatten()
            .cloned()
            .collect())
    }

    // Seems unused
    fn add_ppo(py_self: Py<Self>, py: Python, ppo: Py<PyAny>) -> PyResult<()> {
        let slf = py_self.get();
        let set_res = slf.line_ppos.get_or_try_init(|| {
            slf.refd
                .get("ppos")
                .ok_or_else(|| PyException::new_err("No ppos list"))?
                .downcast_bound::<PyList>(py)?
                .append(ppo)?;
            Err(PyException::new_err("successful insert"))
        });
        match set_res {
            Ok(_) => Err(PyException::new_err("line_ppos already initialized")),
            Err(e) if e.matches(py, PyException::new_err("successful insert")) => Ok(()),
            Err(e) => Err(e),
        }
    }

    // Seems unused
    fn has_ppos(py_self: Py<Self>, py: Python) -> PyResult<bool> {
        Ok(!TestCFunctionRef::ppos(py_self, py)?.is_empty())
    }

    // Seems unused
    fn get_pred_ppos(py_self: Py<Self>, py: Python, pred: &str) -> PyResult<Vec<Py<TestPPORef>>> {
        TestCFunctionRef::ppos(py_self, py)?
            .into_iter()
            .map(|ppo| Ok((ppo.get().predicate(py)?, ppo)))
            .filter_ok(|(ppo_pred, _)| pred == ppo_pred)
            .map_ok(|(_, ppo)| ppo)
            .collect()
    }

    #[getter]
    fn spos(py_self: Py<Self>, py: Python) -> PyResult<Vec<Py<TestSPORef>>> {
        Ok(TestCFunctionRef::line_spos(py_self, py)?
            .values()
            .flatten()
            .cloned()
            .collect())
    }

    // Seems unused
    pub fn has_spos(py_self: Py<Self>, py: Python) -> PyResult<bool> {
        Ok(!TestCFunctionRef::spos(py_self, py)?.is_empty())
    }

    // Seems unused
    fn has_multiple(py_self: Py<Self>, py: Python, line: isize, pred: &str) -> PyResult<bool> {
        Ok(TestCFunctionRef::line_ppos(py_self, py)?
            .get(&line)
            .cloned()
            .unwrap_or_else(|| Vec::new())
            .into_iter()
            .map(|ppo| ppo.get().predicate(py))
            .filter_ok(|ppo_pred| ppo_pred == pred)
            .collect::<PyResult<Vec<String>>>()?
            .len()
            > 1)
    }
}
