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
//! Definition of a struct/union.

use pyo3::{intern, prelude::*};

use crate::{
    app::{
        c_attributes::CAttributes, c_declarations::CDeclarations,
        c_dictionary_record::CDeclarationsRecord, c_field_info::CFieldInfo,
        c_global_declarations::CGlobalDeclarations,
    },
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_comp_info")?;
    module.add_class::<CCompInfo>()?;
    Ok(module)
}

/// Struct/union definition.
///
/// * tags[0]: cname                 ('?' for global struct)
///
/// * args[0]: ckey                  (-1 for global struct)
/// * args[1]: isstruct
/// * args[2]: iattr                 (-1 for global struct)
/// * args[3..]: field indices
#[derive(Clone)]
#[pyclass(extends = CDeclarationsRecord, frozen, subclass)]
pub struct CCompInfo {}

#[pymethods]
impl CCompInfo {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDeclarationsRecord::new(cd, ixval)).add_subclass(CCompInfo {})
    }

    #[getter]
    fn fields<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Vec<Bound<'b, CFieldInfo>>> {
        let decls = slf.borrow().into_super().decls().bind(slf.py()).clone();
        slf.borrow().into_super().into_super().args()[3..]
            .into_iter()
            .map(|i| {
                Ok(decls
                    .call_method1(intern!(slf.py(), "get_fieldinfo"), (*i,))?
                    .downcast()?
                    .clone())
            })
            .collect()
    }

    // Unvalidated
    #[getter]
    fn fieldcount(slf: &Bound<Self>) -> PyResult<isize> {
        Ok(CCompInfo::fields(slf)?.len().try_into()?)
    }

    // Unvalidated
    #[getter]
    fn fieldnames(slf: &Bound<Self>) -> PyResult<Vec<String>> {
        Ok(Self::fields(slf)?
            .into_iter()
            .map(|field| CFieldInfo::fname(field.borrow()))
            .collect())
    }

    #[getter]
    fn is_struct(slf: PyRef<Self>) -> bool {
        slf.into_super().into_super().args()[1] == 1
    }

    // Unvalidated
    #[getter]
    fn cattr<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, CAttributes>> {
        let args_2 = slf.borrow().into_super().into_super().args()[2];
        Ok(slf
            .borrow()
            .into_super()
            .dictionary(slf.py())?
            .bind(slf.py())
            .call_method1(intern!(slf.py(), "get_attributes"), (args_2,))?
            .downcast()?
            .clone())
    }

    #[getter]
    fn ckey(slf: PyRef<Self>) -> isize {
        let it = slf.into_super().into_super();
        if it.args()[0] >= 0 {
            it.args()[0]
        } else {
            it.index()
        }
    }

    // Unvalidated
    #[getter]
    fn size(slf: &Bound<Self>) -> PyResult<isize> {
        Ok(CCompInfo::fields(slf)?
            .iter()
            .map(|field| CFieldInfo::size(field))
            .collect::<PyResult<Vec<isize>>>()?
            .into_iter()
            .sum())
    }

    #[getter]
    fn name(slf: &Bound<Self>) -> PyResult<String> {
        let tag_0 = slf.borrow().into_super().into_super().tags()[0].clone();
        if tag_0 != "?" {
            return Ok(tag_0);
        }
        let global_decls = slf
            .borrow()
            .into_super()
            .decls()
            .bind(slf.py())
            .downcast::<CGlobalDeclarations>()?
            .clone();
        Ok(global_decls
            .call_method1(
                intern!(slf.py(), "compinfo_names"),
                (CCompInfo::ckey(slf.borrow()),),
            )?
            .extract::<Vec<String>>()?[0]
            .clone())
    }

    #[getter]
    fn field_strings(slf: &Bound<Self>) -> PyResult<String> {
        Ok(CCompInfo::fieldnames(slf)?.join(":"))
    }

    // Unvalidated
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        let mut lines = Vec::new();
        lines.push(format!("struct {}", CCompInfo::name(slf)?));
        let mut offset = 0;
        for f in CCompInfo::fields(slf)? {
            lines.push(format!("{}{offset:>4} {}", " ".repeat(5), f.str()?));
            offset += CFieldInfo::size(&f)?;
        }
        Ok(lines.join(""))
    }
}
