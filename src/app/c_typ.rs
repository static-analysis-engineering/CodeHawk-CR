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

use crate::{
    app::{c_dictionary::CDictionary, c_dictionary_record::CDictionaryRecord},
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_typ")?;
    module.add_class::<CTyp>()?;
    Ok(module)
}

/// Base class of all variable types.
#[pyclass(extends = CDictionaryRecord, frozen, subclass)]
pub struct CTyp {}

#[pymethods]
impl CTyp {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDictionaryRecord::new(cd, ixval)).add_subclass(CTyp {})
    }

    fn expand<'a, 'b>(slf: &'a Bound<'b, Self>) -> &'a Bound<'b, Self> {
        slf
    }

    #[getter]
    fn size(&self) -> isize {
        -1000
    }

    #[getter]
    fn is_array(&self) -> bool {
        false
    }

    #[getter]
    fn is_builtin_vaargs(&self) -> bool {
        false
    }

    #[getter]
    fn is_comp(&self) -> bool {
        false
    }

    #[getter]
    fn is_enum(&self) -> bool {
        false
    }

    #[getter]
    fn is_float(&self) -> bool {
        false
    }

    #[getter]
    fn is_function(&self) -> bool {
        false
    }

    #[getter]
    fn is_int(&self) -> bool {
        false
    }

    #[getter]
    fn is_named_type(&self) -> bool {
        false
    }

    #[getter]
    fn is_pointer(&self) -> bool {
        false
    }

    #[getter]
    fn is_struct(&self) -> bool {
        false
    }

    #[getter]
    fn is_void(&self) -> bool {
        false
    }

    #[getter]
    fn is_default_function_prototype(&self) -> bool {
        false
    }
}
