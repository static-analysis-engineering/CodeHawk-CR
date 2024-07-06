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
//! Location in a C source file (filename, line number).

use pyo3::{intern, prelude::*};

use crate::{
    app::{
        c_declarations::CDeclarations, c_dictionary_record::CDeclarationsRecord,
        c_file_declarations::CFileDeclarations,
    },
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_location")?;
    module.add_class::<CLocation>()?;
    Ok(module)
}

/// Location in a C source program.
///
/// - args[0]: filename index
/// - args[1]: byte number
/// - args[2]: line number
#[pyclass(extends = CDeclarationsRecord, frozen, subclass)]
pub struct CLocation {}

#[pymethods]
impl CLocation {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDeclarationsRecord::new(cd, ixval)).add_subclass(CLocation {})
    }

    #[getter]
    fn byte(slf: PyRef<Self>) -> isize {
        slf.into_super().into_super().args()[1]
    }

    #[getter]
    fn line(slf: PyRef<Self>) -> isize {
        slf.into_super().into_super().args()[2]
    }

    #[getter]
    fn file(slf: &Bound<Self>) -> PyResult<String> {
        let args_0 = slf.borrow().into_super().into_super().args()[0];
        let decls = slf
            .borrow()
            .into_super()
            .decls()
            .as_any()
            .downcast_bound::<CFileDeclarations>(slf.py())?
            .clone();
        Ok(decls
            .call_method1(intern!(slf.py(), "get_filename"), (args_0,))?
            .extract()?)
    }

    // Unvalidated
    fn get_loc(slf: &Bound<Self>) -> PyResult<(String, isize, isize)> {
        Ok((
            CLocation::file(slf)?,
            CLocation::line(slf.borrow()),
            CLocation::byte(slf.borrow()),
        ))
    }

    // Unvalidated
    #[pyo3(name = "__ge__")]
    fn ge(slf: &Bound<Self>, loc: &Bound<Self>) -> PyResult<bool> {
        Ok(CLocation::get_loc(slf)? >= CLocation::get_loc(loc)?)
    }

    // Unvalidated
    #[pyo3(name = "__gt__")]
    fn gt(slf: &Bound<Self>, loc: &Bound<Self>) -> PyResult<bool> {
        Ok(CLocation::get_loc(slf)? > CLocation::get_loc(loc)?)
    }

    // Unvalidated
    #[pyo3(name = "__le__")]
    fn le(slf: &Bound<Self>, loc: &Bound<Self>) -> PyResult<bool> {
        Ok(CLocation::get_loc(slf)? <= CLocation::get_loc(loc)?)
    }

    // Unvalidated
    #[pyo3(name = "__lt__")]
    fn lt(slf: &Bound<Self>, loc: &Bound<Self>) -> PyResult<bool> {
        Ok(CLocation::get_loc(slf)? < CLocation::get_loc(loc)?)
    }

    // Unvalidated
    #[pyo3(name = "__eq__")]
    fn eq(slf: &Bound<Self>, loc: &Bound<Self>) -> PyResult<bool> {
        Ok(CLocation::get_loc(slf)? == CLocation::get_loc(loc)?)
    }

    // Unvalidated
    #[pyo3(name = "__ne__")]
    fn ne(slf: &Bound<Self>, loc: &Bound<Self>) -> PyResult<bool> {
        Ok(CLocation::get_loc(slf)? != CLocation::get_loc(loc)?)
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        Ok(format!(
            "{}:{}",
            CLocation::file(slf)?,
            CLocation::line(slf.borrow())
        ))
    }
}
