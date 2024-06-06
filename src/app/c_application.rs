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
use pyo3::{intern, prelude::*};

use crate::app::index_manager::IndexManager;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_application")?;
    module.add_class::<CApplication>()?;
    Ok(module)
}

/// Primary access point for source code and analysis results.
///
/// An application can consist of a single file, or of multiple files managed
/// by a Makefile.
///
/// In case of a single file the following call on CApplication initializes
/// the single file:
///
/// - capp.initialize_single_file(cfilename)
///
/// The filepath in this case is assumed to be empty. The file index is set
/// to 0.
///
/// In case of multiple files the following file is assumed to be present in
/// the top analysis-results directory:
///
/// - <projectpath>/<projectname>.cch/a/target_files.xml
///
/// This file, normally created by the CodeHawk-C parser, is expected to
/// contain a list of c-file entries that provide the attributes:
///
/// - id: a unique file index, a number greater than zero;
/// - name: a string denoting the relative path of the file w.r.t. the project
///   directory (e.g., src/cgi/buffer.c)

#[derive(Clone)]
#[pyclass(subclass)]
pub struct CApplication {
    #[pyo3(get)]
    projectpath: String,
    #[pyo3(get)]
    projectname: String,
    #[pyo3(get)]
    targetpath: String,
    #[pyo3(get)]
    contractpath: String,
    #[pyo3(get)]
    is_singlefile: bool,
    #[pyo3(get)]
    excludefiles: Vec<String>,
    #[pyo3(get)]
    indexmanager: Py<IndexManager>,
}

#[pymethods]
impl CApplication {
    #[new]
    fn new(
        py: Python,
        projectpath: String,
        projectname: String,
        targetpath: String,
        contractpath: String,
        singlefile: Option<bool>,
        excludefiles: Option<Vec<String>>,
    ) -> PyResult<CApplication> {
        let chc = PyModule::import_bound(py, intern!(py, "chc"))?;
        let app = chc.getattr(intern!(py, "app"))?;
        let index_manager_mod = app.getattr(intern!(py, "IndexManager"))?;
        let index_manager_type = index_manager_mod.getattr(intern!(py, "IndexManager"))?;
        let indexmanager = index_manager_type
            .call1((singlefile.unwrap_or(false),))?
            .downcast()?
            .clone()
            .unbind();
        Ok(CApplication {
            projectpath,
            projectname,
            targetpath,
            contractpath,
            is_singlefile: singlefile.unwrap_or(false),
            excludefiles: excludefiles.unwrap_or_else(|| Vec::new()),
            indexmanager,
        })
    }
}
