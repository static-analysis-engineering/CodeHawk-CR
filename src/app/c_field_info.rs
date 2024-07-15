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
//! Type and layout of a struct or union field.

use pyo3::{intern, prelude::*};

use crate::{
    app::{
        c_attributes::CAttributes, c_declarations::CDeclarations,
        c_dictionary_record::CDeclarationsRecord, c_file_declarations::CFileDeclarations,
        c_location::CLocation, c_typ::CTyp,
    },
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_field_info")?;
    module.add_class::<CFieldInfo>()?;
    Ok(module)
}

/// Definition of a struct field.
///
/// * tags[0] fname
///
/// * args[0]: fcomp.ckey  (-1 for global structs)
/// * args[1]: ftype
/// * args[2]: fbitfield
/// * args[3]: fattr       (-1 for global structs)
/// * args[4]: floc        (-1 for global structs)
#[pyclass(extends = CDeclarationsRecord, frozen, subclass)]
pub struct CFieldInfo {}

#[pymethods]
impl CFieldInfo {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDeclarationsRecord::new(cd, ixval)).add_subclass(CFieldInfo {})
    }

    #[getter]
    pub fn fname(slf: PyRef<Self>) -> String {
        slf.into_super().into_super().tags()[0].clone()
    }

    #[getter]
    fn ftype<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, CTyp>> {
        let c_decl_record = slf.borrow().into_super();
        let dictionary = c_decl_record.dictionary(slf.py())?;
        let args_1 = c_decl_record.into_super().args()[1];
        Ok(dictionary
            .call_method1(intern!(slf.py(), "get_typ"), (args_1,))?
            .extract()?)
    }

    #[getter]
    fn bitfield(slf: PyRef<Self>) -> isize {
        slf.into_super().into_super().args()[2]
    }

    // Unvalidated
    #[getter]
    pub fn size(slf: &Bound<Self>) -> PyResult<isize> {
        Ok(CFieldInfo::ftype(slf)?
            .call_method0(intern!(slf.py(), "size"))?
            .extract()?)
    }

    // Unvalidated
    #[getter]
    fn location<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Option<Bound<'b, CLocation>>> {
        let args_4 = slf.borrow().into_super().into_super().args()[4];
        if args_4 < 0 {
            return Ok(None);
        }
        let decls = slf
            .borrow()
            .into_super()
            .decls()
            .as_any()
            .downcast_bound::<CFileDeclarations>(slf.py())?
            .clone();
        Ok(Some(
            decls
                .call_method1(intern!(slf.py(), "get_location"), (args_4,))?
                .extract()?,
        ))
    }

    // Unvalidated
    #[getter]
    fn attributes<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Option<Bound<'b, CAttributes>>> {
        let args_3 = slf.borrow().into_super().into_super().args()[3];
        if args_3 < 0 {
            return Ok(None);
        }
        let dictionary = slf.borrow().into_super().dictionary(slf.py())?;
        Ok(Some(
            dictionary
                .call_method1(intern!(slf.py(), "get_attributes"), (args_3,))?
                .extract()?,
        ))
    }

    // Unvalidated
    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        Ok(format!(
            "{}:{}",
            CFieldInfo::fname(slf.borrow()),
            CFieldInfo::ftype(slf)?.str()?
        ))
    }
}
