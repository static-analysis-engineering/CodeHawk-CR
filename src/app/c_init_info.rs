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
//! Initializer of global variables.

use pyo3::{intern, prelude::*};

use crate::{
    app::{
        c_declarations::CDeclarations, c_dictionary_record::CDeclarationsRecord, c_exp::CExp,
        c_offset::COffset, c_typ::CTyp,
    },
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_init_info")?;
    module.add_class::<CCompoundInitInfo>()?;
    module.add_class::<CInitInfo>()?;
    module.add_class::<CSingleInitInfo>()?;
    module.add_class::<COffsetInitInfo>()?;
    Ok(module)
}

/// Global variable initializer.
#[pyclass(extends = CDeclarationsRecord, frozen, subclass)]
pub struct CInitInfo {}

#[pymethods]
impl CInitInfo {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDeclarationsRecord::new(cd, ixval)).add_subclass(CInitInfo {})
    }

    #[getter]
    fn is_single(&self) -> bool {
        false
    }

    #[getter]
    fn is_compound(&self) -> bool {
        false
    }
}

/// Initializer of a simple variable.
///
/// - args[0]: index of initialization expression in cdictionary
#[pyclass(extends = CInitInfo, frozen, subclass)]
pub struct CSingleInitInfo {}

#[pymethods]
impl CSingleInitInfo {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CInitInfo::new(cd, ixval)).add_subclass(CSingleInitInfo {})
    }

    #[getter]
    fn exp<'a, 'b>(slf: &'a Bound<'b, CSingleInitInfo>) -> PyResult<Bound<'b, CExp>> {
        let c_decl_record = slf.borrow().into_super().into_super();
        let dictionary = c_decl_record.dictionary(slf.py())?;
        let args_0 = c_decl_record.into_super().args()[0];
        Ok(dictionary
            .call_method1(slf.py(), intern!(slf.py(), "get_exp"), (args_0,))?
            .downcast_bound(slf.py())?
            .clone())
    }

    #[getter]
    fn is_single(&self) -> bool {
        true
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        Ok(CSingleInitInfo::exp(slf)?.str()?.extract()?)
    }
}

/// Initializer of a struct or array.
///
/// - args[0]: index of type of initializer in cdictionary
#[pyclass(extends = CInitInfo, frozen, subclass)]
pub struct CCompoundInitInfo {}

// Unvalidated
#[pymethods]
impl CCompoundInitInfo {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CInitInfo::new(cd, ixval)).add_subclass(CCompoundInitInfo {})
    }

    #[getter]
    fn typ<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, CTyp>> {
        let c_decl_record = slf.borrow().into_super().into_super();
        let dictionary = c_decl_record.dictionary(slf.py())?;
        let args_0 = c_decl_record.into_super().args()[0];
        Ok(dictionary
            .call_method1(slf.py(), intern!(slf.py(), "get_typ"), (args_0,))?
            .downcast_bound(slf.py())?
            .clone())
    }

    #[getter]
    fn offset_initializers<'a, 'b>(
        slf: &'a Bound<'b, Self>,
    ) -> PyResult<Vec<Bound<'b, COffsetInitInfo>>> {
        let c_decl_record = slf.borrow().into_super().into_super();
        let decls = c_decl_record.decls();
        c_decl_record.into_super().args()[1..]
            .iter()
            .map(|x| {
                Ok(decls
                    .call_method1(slf.py(), intern!(slf.py(), "get_offset_init"), (*x,))?
                    .downcast_bound(slf.py())?
                    .clone())
            })
            .collect()
    }

    #[getter]
    fn is_compound(&self) -> bool {
        true
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        Ok(CCompoundInitInfo::offset_initializers(slf)?
            .into_iter()
            .map(|x| Ok(x.str()?.extract()?))
            .collect::<PyResult<Vec<String>>>()?
            .join("\n"))
    }
}

/// Component of a compound initializer.
///
/// - args[0]: index of offset expression in cdictionary
/// - args[1]: index of initinfo in cdeclarations
#[pyclass(extends = CDeclarationsRecord, frozen, subclass)]
pub struct COffsetInitInfo {}

// Not validated?
#[pymethods]
impl COffsetInitInfo {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDeclarationsRecord::new(cd, ixval))
            .add_subclass(COffsetInitInfo {})
    }

    #[getter]
    fn offset<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, COffset>> {
        let c_decl_record = slf.borrow().into_super();
        let dictionary = c_decl_record.dictionary(slf.py())?;
        let args_0 = c_decl_record.into_super().args()[0];
        Ok(dictionary
            .call_method1(slf.py(), intern!(slf.py(), "get_offset"), (args_0,))?
            .downcast_bound(slf.py())?
            .clone())
    }

    #[getter]
    fn initializer<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, CInitInfo>> {
        let c_decl_record = slf.borrow().into_super();
        let dictionary = c_decl_record.decls();
        let args_0 = c_decl_record.into_super().args()[1];
        Ok(dictionary
            .call_method1(slf.py(), intern!(slf.py(), "get_initinfo"), (args_0,))?
            .downcast_bound(slf.py())?
            .clone())
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        let offset: String = COffsetInitInfo::offset(slf)?.str()?.extract()?;
        let initializer: String = COffsetInitInfo::initializer(slf)?.str()?.extract()?;
        Ok(format!("{offset}:={initializer}"))
    }
}
