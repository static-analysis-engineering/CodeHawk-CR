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

use pyo3::{prelude::*, types::PyDict};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "index_manager")?;
    module.add_class::<CKeyReference>()?;
    module.add_class::<FileKeyReference>()?;
    module.add_class::<FileVarReference>()?;
    module.add_class::<IndexManager>()?;
    module.add_class::<VarReference>()?;
    Ok(module)
}

// Originally a dataclass, but ordering and hashing weren't used
#[derive(Clone)]
#[pyclass(get_all, set_all)]
pub struct FileVarReference {
    fid: isize, // file index
    vid: isize, // variable index in file with fid
}

#[pymethods]
impl FileVarReference {
    #[new]
    fn new(fid: isize, vid: isize) -> FileVarReference {
        FileVarReference { fid, vid }
    }

    #[getter]
    fn tuple(&self) -> (isize, isize) {
        (self.fid, self.vid)
    }

    #[pyo3(name = "__str__")]
    fn str(&self) -> String {
        format!("(vid: {}, fid: {})", self.vid, self.fid)
    }
}

// Originally a dataclass, but ordering and hasing weren't used
#[derive(Clone)]
#[pyclass(get_all, set_all)]
pub struct FileKeyReference {
    fid: isize,
    ckey: isize,
}

#[pymethods]
impl FileKeyReference {
    #[new]
    fn new(fid: isize, ckey: isize) -> FileKeyReference {
        FileKeyReference { fid, ckey }
    }

    #[pyo3(name = "__str__")]
    fn str(&self) -> String {
        format!("(ckey: {}, fid: {})", self.ckey, self.fid)
    }
}

// Originally a dataclass, but ordering and hasing weren't used
#[derive(Clone)]
#[pyclass(get_all, set_all)]
pub struct VarReference {
    fid: Option<isize>,
    vid: isize,
}

#[pymethods]
impl VarReference {
    #[new]
    #[pyo3(signature = (fid, vid))] // specify the caller must give two arguments
    fn new(fid: Option<isize>, vid: isize) -> VarReference {
        VarReference { fid, vid }
    }

    #[getter]
    fn is_global(&self) -> bool {
        self.fid.is_none()
    }
}

// Originally a dataclass, but ordering and hasing weren't used
#[derive(Clone)]
#[pyclass(get_all, set_all)]
pub struct CKeyReference {
    fid: Option<isize>,
    ckey: isize,
}

#[pymethods]
impl CKeyReference {
    #[new]
    #[pyo3(signature = (fid, ckey))] // specify the caller must give two arguments
    fn new(fid: Option<isize>, ckey: isize) -> CKeyReference {
        CKeyReference { fid, ckey }
    }

    #[getter]
    fn is_global(&self) -> bool {
        self.fid.is_none()
    }
}

#[derive(Clone)]
#[pyclass(get_all, subclass)]
pub struct IndexManager {
    is_single_file: bool,   // application consists of a single file
    vid2gvid: Py<PyDict>,   // fid -> vid -> gvid
    gvid2vid: Py<PyDict>,   // gvid -> fid -> vid
    fidvidmax: Py<PyDict>,  // fid -> maximum vid in file with index fid
    ckey2gckey: Py<PyDict>, // fid -> ckey -> gckey
    gckey2ckey: Py<PyDict>, // gckey -> fid -> ckey
    gviddefs: Py<PyDict>,   // gvid -> fid  (file in which gvid is defined)
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

    // Seems unused
    fn get_vid_gvid_subst(&self, py: Python, fid: isize) -> PyResult<BTreeMap<isize, isize>> {
        Ok(self.vid2gvid.bind(py).as_any().get_item(fid)?.extract()?)
    }

    // Seems unused
    fn get_fid_gvid_subset(
        &self,
        py: Python,
        fileindex: isize,
    ) -> PyResult<BTreeMap<isize, isize>> {
        let mut result = BTreeMap::new();
        for (gvid, table) in self.gvid2vid.bind(py) {
            for (fid, value) in table.downcast::<PyDict>()? {
                if fid.extract::<isize>()? == fileindex {
                    result.insert(gvid.extract()?, value.extract()?);
                }
            }
        }
        Ok(result)
    }
}
