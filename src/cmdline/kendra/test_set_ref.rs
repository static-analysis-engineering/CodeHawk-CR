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

use once_cell::sync::OnceCell;
use pyo3::{exceptions::PyException, intern, prelude::*};

use crate::cmdline::kendra::test_c_file_ref::TestCFileRef;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "test_set_ref")?;
    module.add_class::<TestSetRef>()?;
    Ok(module)
}

/// Provides access to the reference results of a set of C files.
#[derive(Clone)]
#[pyclass(frozen, subclass)]
pub struct TestSetRef {
    #[pyo3(get)]
    specfilename: String,
    refd: OnceCell<BTreeMap<String, Py<PyAny>>>,
    cfiles: OnceCell<BTreeMap<String, Py<TestCFileRef>>>,
}

#[pymethods]
impl TestSetRef {
    #[new]
    fn new(specfilename: String) -> TestSetRef {
        TestSetRef {
            specfilename,
            refd: OnceCell::new(),
            cfiles: OnceCell::new(),
        }
    }

    #[getter]
    fn refd(&self, py: Python) -> PyResult<BTreeMap<String, Py<PyAny>>> {
        self.refd
            .get_or_try_init(|| {
                let builtins = PyModule::import_bound(py, intern!(py, "builtins"))?;
                let fp = builtins
                    .getattr(intern!(py, "open"))?
                    .call1((&self.specfilename,))?;
                let json = PyModule::import_bound(py, intern!(py, "json"))?;
                let refd_any = json.getattr(intern!(py, "load"))?.call1((fp.clone(),))?;
                fp.call_method0(intern!(py, "close"))?;
                refd_any.extract()
            })
            .cloned()
    }

    #[getter]
    fn cfiles(py_slf: Py<Self>, py: Python) -> PyResult<BTreeMap<String, Py<TestCFileRef>>> {
        let slf = py_slf.get();
        slf.cfiles
            .get_or_try_init(|| {
                let refd = slf.refd(py)?;
                let refd_cfiles = refd
                    .get("cfiles")
                    .ok_or_else(|| PyException::new_err("'cfiles' missing"))?;
                let cfiles_dict: BTreeMap<String, Py<PyAny>> = refd_cfiles.extract(py)?;
                let mut cfiles = BTreeMap::new();
                for (f, fdata) in cfiles_dict {
                    cfiles.insert(
                        f.clone(),
                        Py::new(py, TestCFileRef::new(py_slf.clone(), f, fdata.extract(py)?))?,
                    );
                }
                Ok(cfiles)
            })
            .cloned()
    }

    // Seems unused
    #[getter]
    fn cfilenames(py_slf: Py<Self>, py: Python) -> PyResult<Vec<String>> {
        Ok(TestSetRef::cfiles(py_slf, py)?.keys().cloned().collect()) // Sorted by BTreeMap
    }

    // Seems unused
    fn cfile(py_slf: Py<Self>, py: Python, cfilename: &str) -> PyResult<Option<Py<TestCFileRef>>> {
        let cfiles = TestSetRef::cfiles(py_slf, py)?;
        Ok(cfiles.get(cfilename).cloned())
    }

    // Seems unused
    fn has_characteristics(&self, py: Python) -> PyResult<bool> {
        let refd = self.refd(py)?;
        Ok(refd.contains_key("characteristics"))
    }

    // Seems unused
    fn characteristics(&self, py: Python) -> PyResult<Vec<String>> {
        let refd = self.refd(py)?;
        Ok(if let Some(characteristics) = refd.get("characteristics") {
            characteristics.extract(py)?
        } else {
            Vec::new()
        })
    }

    // Seems unused
    fn has_restrictions(&self, py: Python) -> PyResult<bool> {
        let refd = self.refd(py)?;
        Ok(refd.contains_key("restrictions"))
    }

    // Seems unused
    fn restrictions(&self, py: Python) -> PyResult<Vec<String>> {
        let refd = self.refd(py)?;
        Ok(if let Some(restrictions) = refd.get("restrictions") {
            restrictions.extract(py)?
        } else {
            Vec::new()
        })
    }

    // Seems unused
    fn is_linux_only(&self, py: Python) -> PyResult<bool> {
        Ok(self.restrictions(py)?.contains(&"linux-only".to_owned()))
    }
}
