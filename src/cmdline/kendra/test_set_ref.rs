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
use pyo3::{intern, prelude::*, types::PyDict};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "test_set_ref")?;
    module.add_class::<TestSetRef>()?;
    Ok(module)
}

/// Provides access to the reference results of a set of C files.
#[derive(Clone)]
#[pyclass(subclass)]
pub struct TestSetRef {
    #[pyo3(get)]
    specfilename: String,
    #[pyo3(get, set)]
    _refd: Py<PyDict>,
}

#[pymethods]
impl TestSetRef {
    #[new]
    fn new(py: Python, specfilename: String) -> TestSetRef {
        TestSetRef {
            specfilename,
            _refd: PyDict::new_bound(py).unbind(),
        }
    }

    #[getter]
    fn refd(&mut self, py: Python) -> PyResult<Py<PyDict>> {
        if self._refd.bind(py).len() == 0 {
            let builtins = PyModule::import_bound(py, intern!(py, "builtins"))?;
            let fp = builtins
                .getattr(intern!(py, "open"))?
                .call1((&self.specfilename,))?;
            let json = PyModule::import_bound(py, intern!(py, "json"))?;
            let refd_any = json.getattr(intern!(py, "load"))?.call1((fp.clone(),))?;
            let refd_dict = refd_any.downcast()?;
            self._refd = refd_dict.clone().unbind();
            fp.call_method0(intern!(py, "close"))?;
        }
        Ok(self._refd.clone())
    }
}
