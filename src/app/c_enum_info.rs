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
//! Enum declaration data.

use pyo3::{intern, prelude::*};

use crate::{
    app::{
        c_attributes::CAttributes, c_declarations::CDeclarations,
        c_dictionary_record::CDeclarationsRecord, c_enum_item::CEnumItem,
        c_file_declarations::CFileDeclarations,
    },
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_enum_info")?;
    module.add_class::<CEnumInfo>()?;
    Ok(module)
}

/// Global enum definition.
#[derive(Clone)]
#[pyclass(extends = CDeclarationsRecord, frozen, subclass)]
pub struct CEnumInfo {}

#[pymethods]
impl CEnumInfo {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDeclarationsRecord::new(cd, ixval)).add_subclass(CEnumInfo {})
    }

    #[getter]
    fn ename(slf: PyRef<Self>) -> String {
        slf.into_super().into_super().tags()[0].clone()
    }

    // Unvalidated
    #[getter]
    fn ikind(slf: PyRef<Self>) -> String {
        slf.into_super().into_super().tags()[1].clone()
    }

    // Unvalidated
    #[getter]
    fn eattr<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, CAttributes>> {
        let c_decl_record = slf.borrow().into_super();
        let dictionary = c_decl_record.dictionary(slf.py())?;
        let args_0 = c_decl_record.into_super().args()[0];
        Ok(dictionary
            .call_method1(slf.py(), intern!(slf.py(), "get_attributes"), (args_0,))?
            .downcast_bound(slf.py())?
            .clone())
    }

    #[getter]
    fn eitems<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Vec<Bound<'b, CEnumItem>>> {
        let c_dict_record = slf.borrow().into_super();
        let decls = c_dict_record
            .decls()
            .bind(slf.py())
            .downcast::<CFileDeclarations>()?
            .clone();
        c_dict_record.into_super().args()[1..]
            .iter()
            .map(|i| {
                Ok(decls
                    .call_method1(intern!(slf.py(), "get_enumitem"), (*i,))?
                    .downcast::<CEnumItem>()?
                    .clone())
            })
            .collect()
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        Ok(format!(
            "{} ({} items)",
            CEnumInfo::ename(slf.borrow()),
            CEnumInfo::eitems(slf)?.len()
        ))
    }
}
