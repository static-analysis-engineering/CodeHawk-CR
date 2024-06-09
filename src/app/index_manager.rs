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
use pyo3::{prelude::*, types::PyDict};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "index_manager")?;
    module.add_class::<IndexManager>()?;
    Ok(module)
}

#[derive(Clone)]
#[pyclass(subclass)]
pub struct IndexManager {
    #[pyo3(get)]
    is_single_file: bool, // application consists of a single file
    #[pyo3(get)]
    vid2gvid: Py<PyDict>, // fid -> vid -> gvid
    #[pyo3(get)]
    gvid2vid: Py<PyDict>, // gvid -> fid -> vid
    #[pyo3(get)]
    fidvidmax: Py<PyDict>, // fid -> maximum vid in file with index fid
    #[pyo3(get)]
    ckey2gckey: Py<PyDict>, // fid -> ckey -> gckey
    #[pyo3(get)]
    gckey2ckey: Py<PyDict>, // gckey -> fid -> ckey
    #[pyo3(get)]
    gviddefs: Py<PyDict>, // gvid -> fid  (file in which gvid is defined)
}

#[pymethods]
impl IndexManager {
    #[new]
    fn new(py: Python, issinglefile: bool) -> IndexManager {
        IndexManager {
            is_single_file: issinglefile,
            vid2gvid: PyDict::new_bound(py).unbind(),
            gvid2vid: PyDict::new_bound(py).unbind(),
            fidvidmax: PyDict::new_bound(py).unbind(),
            ckey2gckey: PyDict::new_bound(py).unbind(),
            gckey2ckey: PyDict::new_bound(py).unbind(),
            gviddefs: PyDict::new_bound(py).unbind(),
        }
    }
}
