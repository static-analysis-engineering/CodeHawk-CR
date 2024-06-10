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

use pyo3::{intern, prelude::*, types::PyDict};

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

fn chklogger_debug(py: Python, text: String) -> PyResult<()> {
    let chc = PyModule::import_bound(py, intern!(py, "chc"))?;
    let util = chc.getattr(intern!(py, "util"))?;
    let loggingutil = util.getattr(intern!(py, "loggingutil"))?;
    let chklogger = loggingutil.getattr(intern!(py, "chklogger"))?;
    let logger = chklogger.getattr(intern!(py, "logger"))?;
    logger.call_method1(intern!(py, "debug"), (text,))?;
    Ok(())
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

    /// Returns the local reference of the definition of (fid, vid).
    ///
    /// An object (variable or function) may be declared in one file (fid) and referenced by vid,
    /// but defined in another file, with file index def-fid and variable reference def-vid. If the
    /// definition is found this method returns (def-fid, def-vid).
    fn resolve_vid(
        &mut self,
        py: Python,
        filevar: FileVarReference,
    ) -> PyResult<Option<FileVarReference>> {
        if self.is_single_file {
            return Ok(Some(filevar)); // there is only one file, so all objects must be defined there.
        }
        let (fid, vid) = filevar.tuple();
        let Some(vid2gvid_fid) = self.vid2gvid.bind(py).get_item(fid)? else {
            chklogger_debug(py, format!("file id {fid} not found in vid2gvid"))?;
            return Ok(None);
        };
        let vid2gvid_fid = vid2gvid_fid.downcast::<PyDict>()?;
        let Some(gvid) = vid2gvid_fid.get_item(vid)? else {
            chklogger_debug(
                py,
                format!("local vid {vid} not found in vid2gvid[{fid}] for ({fid}, {vid})"),
            )?;
            return Ok(None);
        };
        let gvid: isize = gvid.extract()?;
        let Some(deffid) = self.gviddefs.bind(py).get_item(gvid)? else {
            chklogger_debug(
                py,
                format!("global vid {gvid} not found gviddefs for ({fid}, {vid})"),
            )?;
            return Ok(None);
        };
        let deffid: isize = deffid.extract()?;
        let Some(gvid2vid_gvid) = self.gvid2vid.bind(py).get_item(gvid)? else {
            chklogger_debug(
                py,
                format!("global vid {gvid} not found in gvid2vid for ({fid}, {vid})"),
            )?;
            return Ok(None);
        };
        let gvid2vid_gvid = gvid2vid_gvid.downcast::<PyDict>()?;
        let Some(defvid) = gvid2vid_gvid.get_item(deffid)? else {
            chklogger_debug(
                py,
                format!("target fid: {deffid} not found in gvid2vid[{gvid}] for ({fid}, {vid})"),
            )?;
            return Ok(None);
        };
        let defvid: isize = defvid.extract()?;
        Ok(Some(FileVarReference::new(deffid, defvid)))
    }

    // Seems unused
    /// Returns a list all file variables that refer to the same global var.
    fn get_gvid_references(&self, py: Python, gvid: isize) -> PyResult<Vec<Py<FileVarReference>>> {
        let Some(gvid2vid_gvid) = self.gvid2vid.bind(py).get_item(gvid)? else {
            return Ok(vec![]);
        };
        let gvid2vid_gvid = gvid2vid_gvid.downcast::<PyDict>()?;
        gvid2vid_gvid
            .into_iter()
            .map(|(fid, vid)| Py::new(py, FileVarReference::new(fid.extract()?, vid.extract()?)))
            .collect()
    }

    // Seems unused
    fn has_gvid_reference(&self, py: Python, gvid: isize, fid: isize) -> PyResult<bool> {
        let Some(gvid2vid_gvid) = self.gvid2vid.bind(py).get_item(gvid)? else {
            return Ok(false);
        };
        let gvid2vid_gvid = gvid2vid_gvid.downcast::<PyDict>()?;
        gvid2vid_gvid.contains(fid)
    }

    // Seems unused
    /// Returns the vid that corresponds to gvid in the file with index fid.
    fn get_gvid_reference(&self, py: Python, gvid: isize, fid: isize) -> PyResult<Option<isize>> {
        let Some(gvid2vid_gvid) = self.gvid2vid.bind(py).get_item(gvid)? else {
            return Ok(None);
        };
        let gvid2vid_gvid = gvid2vid_gvid.downcast::<PyDict>()?;
        if let Some(ret) = gvid2vid_gvid.get_item(fid)? {
            Ok(Some(ret.extract()?))
        } else {
            Ok(None)
        }
    }

    // Seems unused
    /// Returns a list of file vars that refer to the same variable as filevar.
    ///
    /// Note: does not include filevar itself.
    fn get_vid_references(
        &self,
        py: Python,
        filevar: Bound<FileVarReference>,
    ) -> PyResult<Vec<Py<FileVarReference>>> {
        if self.is_single_file {
            return Ok(vec![]);
        }
        let Some(vid2gvid_fid) = self.vid2gvid.bind(py).get_item(filevar.borrow().fid)? else {
            return Ok(vec![]);
        };
        let vid2gvid_fid = vid2gvid_fid.downcast::<PyDict>()?;
        let Some(gvid) = vid2gvid_fid.get_item(filevar.borrow().vid)? else {
            return Ok(vec![]);
        };
        let gvid: isize = gvid.extract()?;
        let Some(gvid2vid_gvid) = self.gvid2vid.bind(py).get_item(gvid)? else {
            return Ok(vec![]);
        };
        let gvid2vid_gvid = gvid2vid_gvid.downcast::<PyDict>()?;
        gvid2vid_gvid
            .into_iter()
            .filter(|(fid, _)| fid.extract().unwrap_or(-1) != filevar.borrow().fid)
            .map(|(fid, vid)| Py::new(py, FileVarReference::new(fid.extract()?, vid.extract()?)))
            .collect()
    }

    /// Returns the global vid that corresponds to the file var reference.
    fn get_gvid(&self, py: Python, varref: Bound<FileVarReference>) -> PyResult<Option<isize>> {
        let varref = varref.borrow();
        if self.is_single_file {
            return Ok(Some(varref.vid));
        }
        let Some(vid2gvid_fid) = self.vid2gvid.bind(py).get_item(varref.fid)? else {
            return Ok(None);
        };
        let vid2gvid_fid = vid2gvid_fid.downcast::<PyDict>()?;
        if let Some(gvid) = vid2gvid_fid.get_item(varref.vid)? {
            Ok(Some(gvid.extract()?))
        } else {
            Ok(None)
        }
    }
}
