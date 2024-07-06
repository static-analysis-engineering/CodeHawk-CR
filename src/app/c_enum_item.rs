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
//! Enum item declaration data.

use pyo3::{intern, prelude::*};

use crate::{
    app::{
        c_declarations::CDeclarations, c_dictionary_record::CDeclarationsRecord, c_exp::CExp,
        c_file_declarations::CFileDeclarations, c_location::CLocation,
    },
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_enum_item")?;
    module.add_class::<CEnumItem>()?;
    Ok(module)
}

/// Enum Item.
///
/// * tags[0]: name of the item
/// * args[0]: index of expression associated with the item in cdictionary
/// * args[1]: index of definition location in the declarations
#[pyclass(extends = CDeclarationsRecord, frozen, subclass)]
pub struct CEnumItem {}

#[pymethods]
impl CEnumItem {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDeclarationsRecord::new(cd, ixval)).add_subclass(CEnumItem {})
    }

    #[getter]
    fn name(slf: PyRef<Self>) -> String {
        slf.into_super().into_super().tags()[0].clone()
    }

    #[getter]
    fn exp<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, CExp>> {
        let c_dict_record = slf.borrow().into_super();
        let dictionary = c_dict_record.dictionary(slf.py())?;
        let args_0 = c_dict_record.into_super().args()[0];
        Ok(dictionary
            .call_method1(slf.py(), intern!(slf.py(), "get_exp"), (args_0,))?
            .downcast_bound(slf.py())?
            .clone())
    }

    // Unvalidated
    #[getter]
    fn loc<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, CLocation>> {
        let c_dict_record = slf.borrow().into_super();
        let decls = c_dict_record
            .decls()
            .bind(slf.py())
            .downcast::<CFileDeclarations>()?
            .clone();
        let args_1 = c_dict_record.into_super().args()[1];
        Ok(decls
            .call_method1(intern!(slf.py(), "get_location"), (args_1,))?
            .downcast()?
            .clone())
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        Ok(format!(
            "{}:{}",
            CEnumItem::name(slf.borrow()),
            CEnumItem::exp(slf)?.str()?
        ))
    }
}
