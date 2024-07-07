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
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use once_cell::sync::OnceCell;
use pyo3::{intern, prelude::*};

use crate::app::c_application::CApplication;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_src_file")?;
    module.add_class::<CSrcFile>()?;
    Ok(module)
}

fn chklogger_warning(py: Python, text: String) -> PyResult<()> {
    let chc = PyModule::import_bound(py, intern!(py, "chc"))?;
    let util = chc.getattr(intern!(py, "util"))?;
    let loggingutil = util.getattr(intern!(py, "loggingutil"))?;
    let chklogger = loggingutil.getattr(intern!(py, "chklogger"))?;
    let logger = chklogger.getattr(intern!(py, "logger"))?;
    logger.call_method1(intern!(py, "warning"), (text,))?;
    Ok(())
}

/// Represents the text file that holds the C source code.
#[pyclass(frozen, subclass)]
pub struct CSrcFile {
    #[pyo3(get)]
    capp: Py<CApplication>,
    /// Returns the absolute c filename relative to the project directory.
    #[pyo3(get)]
    fname: String,
    lines: OnceCell<BTreeMap<usize, String>>,
}

impl CSrcFile {
    fn lines(&self, py: Python) -> PyResult<&BTreeMap<usize, String>> {
        self.lines.get_or_try_init(|| {
            let path = Path::new(&self.fname);
            if !path.exists() {
                chklogger_warning(py, format!("Source file {} was not found", self.fname))?;
                return Ok(BTreeMap::new());
            }
            BufReader::new(File::open(path)?)
                .lines()
                .enumerate()
                .map(|(n, io_res)| Ok((n + 1, io_res?)))
                .collect()
        })
    }
}

#[pymethods]
impl CSrcFile {
    #[new]
    fn new(capp: Py<CApplication>, fname: String) -> CSrcFile {
        CSrcFile {
            capp,
            fname,
            lines: OnceCell::new(),
        }
    }

    #[getter]
    #[pyo3(name = "lines")]
    fn cloned_lines(&self, py: Python) -> PyResult<BTreeMap<usize, String>> {
        self.lines(py).cloned()
    }

    fn get_line_count(&self, py: Python) -> PyResult<usize> {
        Ok(self.lines(py)?.len())
    }

    fn get_line(&self, py: Python, n: usize) -> PyResult<Option<String>> {
        Ok(self.lines(py)?.get(&n).cloned())
    }
}
