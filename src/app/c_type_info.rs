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
//! Global type definition.

use pyo3::{intern, prelude::*};

use crate::{
    app::{c_declarations::CDeclarations, c_dictionary_record::CDeclarationsRecord, c_typ::CTyp},
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_type_info")?;
    module.add_class::<CTypeInfo>()?;
    Ok(module)
}

/// Type definition.
///
/// - tags[0]: name of type definition
/// - args[1]: index of type of type definition in cdictionary
#[derive(Clone)]
#[pyclass(extends = CDeclarationsRecord, frozen, subclass)]
pub struct CTypeInfo {}

#[pymethods]
impl CTypeInfo {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDeclarationsRecord::new(cd, ixval)).add_subclass(CTypeInfo {})
    }

    #[getter]
    fn name(slf: PyRef<Self>) -> String {
        slf.into_super().into_super().tags()[0].clone()
    }

    // Seems unused
    #[getter]
    fn r#type<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, CTyp>> {
        let c_dict_record = slf.borrow().into_super();
        let dict = c_dict_record.dictionary(slf.py())?;
        // Comment says args[1] but original source uses args[0]
        let args_0 = c_dict_record.into_super().args()[0];
        Ok(dict
            .call_method1(slf.py(), intern!(slf.py(), "get_typ"), (args_0,))?
            .downcast_bound(slf.py())?
            .clone())
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        Ok(format!(
            "{}:{}",
            CTypeInfo::name(slf.borrow()),
            CTypeInfo::r#type(slf)?.str()?
        ))
    }
}
