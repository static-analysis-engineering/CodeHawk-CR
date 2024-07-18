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

use crate::app::c_application::CApplication;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "analysis_manager")?;
    module.add_class::<AnalysisManager>()?;
    Ok(module)
}

/// Provide the interface to the codehawk (ocaml) analyzer.
#[pyclass(get_all, frozen, subclass)]
pub struct AnalysisManager {
    capp: Py<CApplication>,
    wordsize: isize,
    unreachability: bool,
    thirdpartysummaries: Vec<String>,
    nofilter: bool,
    verbose: bool,
}

#[pymethods]
impl AnalysisManager {
    /// Initialize the analyzer location and target file location.
    ///
    /// Args:
    ///     capp (CApplication): application entry point
    ///
    /// Keyword args:
    ///     wordsize (int): architecture wordsize (0,16,32,64) (default 0 (unspecified))
    ///     unreachability (bool): use unreachability as justification to discharge
    ///                            (default False)
    ///     thirdpartysummaries (string list): names of function summary jars
    ///     verbose (bool): display analyzer output (default True)
    ///     nofilter (bool): don't remove functions with absolute filename (default True)
    #[new]
    #[pyo3(signature = (capp, wordsize = 0, unreachability = false, thirdpartysummaries = vec![], nofilter = true, verbose = false))]
    fn new(
        capp: Py<CApplication>,
        wordsize: isize,
        unreachability: bool,
        thirdpartysummaries: Vec<String>,
        nofilter: bool,
        verbose: bool,
    ) -> AnalysisManager {
        AnalysisManager {
            capp,
            wordsize,
            unreachability,
            thirdpartysummaries,
            nofilter,
            verbose,
        }
    }
}
