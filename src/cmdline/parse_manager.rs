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
use pyo3::prelude::*;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "parse_manager")?;
    module.add_class::<ParseManager>()?;
    Ok(module)
}

/// Utility functions to support preprocessing and parsing source code.
///
/// Naming conventions:
///
/// - cfilename     base name of cfile analyzed (without extension)
/// - cfilename_c   idem, with .c extension
/// - projectpath   full-path in which cfilename_c resides (in case of a
///                   single file analyzed) or in which the Makefile of
///                   the project resides (in case of a multi-file project)
/// - targetpath    full-path of directory in which results are saved
/// - projectname   name under which results are saved
///
/// Auxiliary names:
///
/// - cchpath       full-path of analysis results (targetpath/projectname.cch)
/// - cchname       base name of cchpath (projectname.cch)
/// - cchtarname    projectname.cch.tar
/// - cchtargzname  projectname.cch.tar.gz
/// - cchtarfile    targetpath/projectname.cch.tar
/// - cchtargzfile  targetpath/projectname.cch.tar.gz
#[pyclass(subclass)]
pub struct ParseManager {}

#[pymethods]
impl ParseManager {
    /// Initialize paths to code, results, and parser executable.
    ///
    /// Args:
    ///     cpath: absolute path to toplevel C source directory
    ///     tgtpath: absolute path to analysis directory
    ///     sempathname: local name of semantics directory
    ///
    /// Effects:
    ///     creates tgtpath and subdirectories if necessary.
    #[new]
    fn new() -> ParseManager {
        ParseManager {}
    }
}
