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
//! Object representation for CIL offset sum type.

use std::collections::BTreeMap;

use pyo3::{exceptions::PyException, intern, prelude::*};

use crate::{
    app::{
        c_dictionary::CDictionary,
        c_dictionary_record::{CDictionaryRecord, CDictionaryRegistryEntry},
        c_exp::CExp,
    },
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_offset")?;
    module.add_class::<COffset>()?;
    Ok(module)
}

// Needs to be a separate type because of #[pyclass]
#[derive(Clone)]
enum COffsetType {
    CNoOffset,
    /// Field offset
    ///
    /// * tags[1]: fieldname
    ///
    /// * args[0]: ckey of the containing struct
    /// * args[1]: index of sub-offset in cdictionary
    CFieldOffset,
    /// Index offset into an array.
    ///
    /// * args[0]: index of base of index expression in cdictionary
    /// * args[1]: index of sub-offset in cdictionary
    CIndexOffset,
}

/// Base class for an expression offset.
#[derive(Clone)]
#[pyclass(extends = CDictionaryRecord, frozen, subclass)]
pub struct COffset {
    typ: COffsetType,
}

impl COffset {
    fn new(
        typ: COffsetType,
        cd: Py<CDictionary>,
        ixval: IndexedTableValue,
    ) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDictionaryRecord::new(cd, ixval)).add_subclass(COffset { typ })
    }
}

#[pymethods]
impl COffset {
    fn has_offset(&self) -> bool {
        !matches!(self.typ, COffsetType::CNoOffset)
    }

    #[getter]
    fn is_no_offset(&self) -> bool {
        matches!(self.typ, COffsetType::CNoOffset)
    }

    #[getter]
    fn is_field(&self) -> bool {
        matches!(self.typ, COffsetType::CFieldOffset)
    }

    #[getter]
    fn is_index(&self) -> bool {
        matches!(self.typ, COffsetType::CIndexOffset)
    }

    fn get_strings(slf: &Bound<Self>) -> PyResult<Vec<String>> {
        Ok(match slf.borrow().typ {
            COffsetType::CIndexOffset => {
                let index_exp = Self::index_exp(slf)?;
                // Resolve with python interpreter in case this method is overridden
                index_exp
                    .call_method0(intern!(slf.py(), "get_strings"))?
                    .extract()?
            }
            _ => vec![],
        })
    }

    fn get_variable_uses(slf: &Bound<Self>, vid: isize) -> PyResult<isize> {
        Ok(match slf.borrow().typ {
            COffsetType::CIndexOffset => {
                let index_exp = Self::index_exp(slf)?;
                // Resolve with python interpreter in case this method is overridden
                index_exp
                    .call_method1(intern!(slf.py(), "get_variable_uses"), (vid,))?
                    .extract()?
            }
            _ => 0,
        })
    }

    // Unvalidated
    fn to_dict(slf: &Bound<Self>) -> PyResult<BTreeMap<String, Py<PyAny>>> {
        let py = slf.py();
        Ok(match slf.borrow().typ {
            COffsetType::CNoOffset => {
                BTreeMap::from([("base".to_string(), "no-offset".to_string().to_object(py))])
            }
            COffsetType::CFieldOffset => {
                let mut map = BTreeMap::from([
                    ("base".to_string(), "field-offset".to_object(py)),
                    ("field".to_string(), Self::fieldname(slf)?.to_object(py)),
                ]);
                let offset = Self::offset(slf)?;
                // Resolve call with python interpreter for possible override
                if offset
                    .call_method0(intern!(slf.py(), "has_offset"))?
                    .extract()?
                {
                    let inner = offset.call_method0(intern!(slf.py(), "to_dict"))?;
                    map.insert("offset".to_string(), inner.unbind());
                }
                map
            }
            COffsetType::CIndexOffset => {
                let mut map = BTreeMap::from([
                    ("base".to_string(), "field-offset".to_object(py)),
                    (
                        "field".to_string(),
                        Self::index_exp(slf)?
                            .call_method0(intern!(py, "to_dict"))?
                            .unbind(),
                    ),
                ]);
                let offset = Self::offset(slf)?;
                // Resolve call with python interpreter for possible override
                if offset
                    .call_method0(intern!(slf.py(), "has_offset"))?
                    .extract()?
                {
                    let inner = offset.call_method0(intern!(slf.py(), "to_dict"))?;
                    map.insert("offset".to_string(), inner.unbind());
                }
                map
            }
        })
    }

    #[getter]
    fn offset<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, COffset>> {
        let py = slf.py();
        let c_dict_record = slf.borrow().into_super();
        let cd = c_dict_record.cd();
        Ok(match slf.borrow().typ {
            COffsetType::CFieldOffset => {
                let arg_1 = c_dict_record.into_super().args()[1];
                cd.call_method1(py, intern!(py, "get_offset"), (arg_1,))?
                    .downcast_bound(py)?
                    .clone()
            }
            COffsetType::CIndexOffset => {
                let arg_1 = c_dict_record.into_super().args()[1];
                // Resolve with python interpreter in case this method is overridden
                cd.call_method1(py, intern!(py, "get_offset"), (arg_1,))?
                    .downcast_bound(py)?
                    .clone()
            }
            _ => return Err(PyException::new_err("wrong type")),
        })
    }

    // Unvalidated
    #[getter]
    fn index_exp<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Bound<'b, CExp>> {
        if !matches!(slf.borrow().typ, COffsetType::CIndexOffset) {
            return Err(PyException::new_err("wrong type"));
        }
        let py = slf.py();
        let c_dict_record = slf.borrow().into_super();
        let cd = c_dict_record.cd();
        let arg_0 = c_dict_record.into_super().args()[0];
        // Resolve with python interpreter in case this method is overridden
        Ok(cd
            .call_method1(py, intern!(py, "get_exp"), (arg_0,))?
            .downcast_bound(py)?
            .clone())
    }

    #[getter]
    fn fieldname(slf: &Bound<Self>) -> PyResult<String> {
        if !matches!(slf.borrow().typ, COffsetType::CFieldOffset) {
            return Err(PyException::new_err("wrong type"));
        }
        Ok(slf.borrow().into_super().into_super().tags()[1].clone())
    }

    #[getter]
    fn ckey(slf: &Bound<Self>) -> PyResult<isize> {
        if !matches!(slf.borrow().typ, COffsetType::CFieldOffset) {
            return Err(PyException::new_err("wrong type"));
        }
        Ok(slf.borrow().into_super().into_super().args()[0])
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        match slf.borrow().typ {
            COffsetType::CNoOffset => Ok("".to_string()),
            COffsetType::CFieldOffset => {
                // Resolve call with python interpret for possible override
                let offset = if slf
                    .call_method0(intern!(slf.py(), "has_offset"))?
                    .extract()?
                {
                    Self::offset(slf)?.str()?.extract()?
                } else {
                    "".to_string()
                };
                Ok(format!(".{}{offset}", Self::fieldname(slf)?))
            }
            COffsetType::CIndexOffset => {
                // Resolve call with python interpret for possible override
                let offset = if slf
                    .call_method0(intern!(slf.py(), "has_offset"))?
                    .extract()?
                {
                    Self::offset(slf)?.str()?.extract()?
                } else {
                    "".to_string()
                };
                Ok(format!("[{}]{offset}", Self::index_exp(slf)?.str()?))
            }
        }
    }
}

fn create_no_offset(
    cd: &Bound<CDictionary>,
    ixval: &Bound<IndexedTableValue>,
) -> PyResult<Py<CDictionaryRecord>> {
    Ok(Bound::new(
        cd.py(),
        COffset::new(
            COffsetType::CNoOffset,
            cd.clone().unbind(),
            ixval.clone().unbind().get().clone(),
        ),
    )?
    .downcast()?
    .clone()
    .unbind())
}

inventory::submit! { CDictionaryRegistryEntry::rust_type::<COffset>("n", &create_no_offset) }

fn create_field_offset(
    cd: &Bound<CDictionary>,
    ixval: &Bound<IndexedTableValue>,
) -> PyResult<Py<CDictionaryRecord>> {
    Ok(Bound::new(
        cd.py(),
        COffset::new(
            COffsetType::CFieldOffset,
            cd.clone().unbind(),
            ixval.clone().unbind().get().clone(),
        ),
    )?
    .downcast()?
    .clone()
    .unbind())
}

inventory::submit! { CDictionaryRegistryEntry::rust_type::<COffset>("f", &create_field_offset) }

fn create_index_offset(
    cd: &Bound<CDictionary>,
    ixval: &Bound<IndexedTableValue>,
) -> PyResult<Py<CDictionaryRecord>> {
    Ok(Bound::new(
        cd.py(),
        COffset::new(
            COffsetType::CIndexOffset,
            cd.clone().unbind(),
            ixval.clone().unbind().get().clone(),
        ),
    )?
    .downcast()?
    .clone()
    .unbind())
}

inventory::submit! { CDictionaryRegistryEntry::rust_type::<COffset>("i", &create_index_offset) }
