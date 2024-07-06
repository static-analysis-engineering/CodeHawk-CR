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

use crate::app::c_declarations::CDeclarations;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_global_declarations")?;
    module.add_class::<CGlobalDeclarations>()?;
    Ok(module)
}
/// Dictionary that indexes global vars and struct definitions from all files.
///
/// The indexing of struct definitions may involve backtracking in the case of
/// structs that contain pointer references to itself, or circular references
/// that involve multiple structs.
///
/// The backtracking is performed per file. When a struct (represented by a
/// compinfo) is indexed its status is set to pending. When a request for a
/// TComp ckey conversion for the same compinfo is encountered a new global
/// key is conjectured as follows:
///
/// - gckey that has already been reserved for this ckey
/// - gckey that has already been conjectured for this ckey
/// - gckey for an existing global compinfo that has the same fields, if
///   (ckey,gckey) is not in the list of incompatibles
/// - reserve a new key from the indexed table and set its status to reserved,
///   and remove its pending status
///
/// When the compinfo for ckey has been constructed the state is updated as
/// follows:
///
/// - if ckey had a reserved key the reserved key is now committed
/// - if ckey had a conjectured key and the conjectured key is the same as the
///   returned gckey, nothing needs to be done
/// - if ckey had a conjectured key but the conjectured key is not the same as the
///   returned gckey, add (ckey,gckey) to the list of incompatibles, reset
///   the indexed table to the file checkpoint, and re-index all compinfos
///   in the file.
#[pyclass(extends = CDeclarations, frozen, subclass)]
pub struct CGlobalDeclarations {}

#[pymethods]
impl CGlobalDeclarations {
    #[new]
    fn new() -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDeclarations::new()).add_subclass(CGlobalDeclarations {})
    }
}
