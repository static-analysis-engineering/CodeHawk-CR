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
enum COffsetType {
    CNoOffset,
    /// Field offset
    ///
    /// * tags[1]: fieldname
    ///
    /// * args[0]: ckey of the containing struct
    /// * args[1]: index of sub-offset in cdictionary
    CFieldOffset {
        fieldname: String,
        ckey: isize,
        index: isize,
    },
    /// Index offset into an array.
    ///
    /// * args[0]: index of base of index expression in cdictionary
    /// * args[1]: index of sub-offset in cdictionary
    CIndexOffset {
        base_index: isize,
        sub_offset_index: isize,
    },
}

/// Base class for an expression offset.
#[pyclass(extends = CDictionaryRecord, frozen)]
pub struct COffset {
    cd: Py<CDictionary>,
    typ: COffsetType,
}

impl COffset {
    fn new(
        typ: COffsetType,
        cd: &Bound<CDictionary>,
        ixval: IndexedTableValue,
    ) -> PyClassInitializer<Self> {
        let cd = cd.clone().unbind();
        PyClassInitializer::from(CDictionaryRecord::new(cd.clone(), ixval))
            .add_subclass(COffset { cd, typ })
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
        matches!(self.typ, COffsetType::CFieldOffset { .. })
    }

    #[getter]
    fn is_index(&self) -> bool {
        matches!(self.typ, COffsetType::CIndexOffset { .. })
    }

    fn get_strings(&self, py: Python) -> PyResult<Vec<String>> {
        if !matches!(self.typ, COffsetType::CIndexOffset { .. }) {
            return Ok(vec![]);
        }
        // Resolve with python interpreter in case this method is overridden
        Ok(self
            .index_exp(py)?
            .call_method0(intern!(py, "get_strings"))?
            .extract()?)
    }

    fn get_variable_uses(&self, py: Python, vid: isize) -> PyResult<isize> {
        if !matches!(self.typ, COffsetType::CIndexOffset { .. }) {
            return Ok(0);
        }
        // Resolve with python interpreter in case this method is overridden
        Ok(self
            .index_exp(py)?
            .call_method1(intern!(py, "get_variable_uses"), (vid,))?
            .extract()?)
    }

    // Unvalidated
    fn to_dict(&self, py: Python) -> PyResult<BTreeMap<&'static str, Py<PyAny>>> {
        let mut map = BTreeMap::new();
        if self.has_offset() {
            let inner = self.offset(py)?;
            if inner.get().has_offset() {
                map.insert("offset", inner.get().to_dict(py)?.to_object(py));
            }
        }
        match self.typ {
            COffsetType::CNoOffset => {
                map.insert("base", "no-offset".to_object(py));
            }
            COffsetType::CFieldOffset { .. } => {
                map.insert("base", "field-offset".to_object(py));
                map.insert("field", self.fieldname()?.to_object(py));
            }
            COffsetType::CIndexOffset { .. } => {
                map.insert("base", "field-offset".to_object(py));
                map.insert(
                    "field",
                    self.index_exp(py)?
                        .call_method0(intern!(py, "to_dict"))?
                        .unbind(),
                );
            }
        };
        Ok(map)
    }

    #[getter]
    fn offset<'a, 'b>(&'a self, py: Python<'b>) -> PyResult<Bound<'b, COffset>> {
        let index = match self.typ {
            COffsetType::CFieldOffset { index, .. } => index,
            COffsetType::CIndexOffset {
                sub_offset_index, ..
            } => sub_offset_index,
            _ => return Err(PyException::new_err("wrong type")),
        };
        // Resolve with python interpreter in case this method is overridden
        Ok(self
            .cd
            .call_method1(py, intern!(py, "get_offset"), (index,))?
            .downcast_bound(py)?
            .clone())
    }

    // Unvalidated
    #[getter]
    fn index_exp<'a, 'b>(&'a self, py: Python<'b>) -> PyResult<Bound<'b, CExp>> {
        let COffsetType::CIndexOffset { base_index, .. } = self.typ else {
            return Err(PyException::new_err("wrong type"));
        };
        // Resolve with python interpreter in case this method is overridden
        Ok(self
            .cd
            .call_method1(py, intern!(py, "get_exp"), (base_index,))?
            .downcast_bound(py)?
            .clone())
    }

    #[getter]
    fn fieldname(&self) -> PyResult<String> {
        let COffsetType::CFieldOffset { fieldname, .. } = &self.typ else {
            return Err(PyException::new_err("wrong type"));
        };
        Ok(fieldname.clone())
    }

    #[getter]
    fn ckey(&self) -> PyResult<isize> {
        let COffsetType::CFieldOffset { ckey, .. } = &self.typ else {
            return Err(PyException::new_err("wrong type"));
        };
        Ok(*ckey)
    }

    #[pyo3(name = "__str__")]
    fn str(&self, py: Python) -> PyResult<String> {
        match self.typ {
            COffsetType::CNoOffset => Ok("".to_string()),
            COffsetType::CFieldOffset { .. } => {
                let offset: String = self.offset(py)?.str()?.extract()?;
                Ok(format!(".{}{offset}", self.fieldname()?))
            }
            COffsetType::CIndexOffset { .. } => {
                let offset = if self.has_offset() {
                    self.offset(py)?.str()?.extract()?
                } else {
                    "".to_string()
                };
                Ok(format!("[{}]{offset}", self.index_exp(py)?.str()?))
            }
        }
    }
}

fn create_no_offset<'a, 'b>(
    cd: &'a Bound<'b, CDictionary>,
    ixval: IndexedTableValue,
) -> PyResult<Bound<'b, CDictionaryRecord>> {
    Ok(
        Bound::new(cd.py(), COffset::new(COffsetType::CNoOffset, cd, ixval))?
            .downcast()?
            .clone(),
    )
}

inventory::submit! { CDictionaryRegistryEntry::rust_type::<COffset>("n", &create_no_offset) }

fn create_field_offset<'a, 'b>(
    cd: &'a Bound<'b, CDictionary>,
    ixval: IndexedTableValue,
) -> PyResult<Bound<'b, CDictionaryRecord>> {
    let typ = COffsetType::CFieldOffset {
        fieldname: ixval.tags()[1].clone(),
        ckey: ixval.args()[0],
        index: ixval.args()[1],
    };
    Ok(Bound::new(cd.py(), COffset::new(typ, cd, ixval))?
        .downcast()?
        .clone())
}

inventory::submit! { CDictionaryRegistryEntry::rust_type::<COffset>("f", &create_field_offset) }

fn create_index_offset<'a, 'b>(
    cd: &'a Bound<'b, CDictionary>,
    ixval: IndexedTableValue,
) -> PyResult<Bound<'b, CDictionaryRecord>> {
    let typ = COffsetType::CIndexOffset {
        base_index: ixval.args()[0],
        sub_offset_index: ixval.args()[1],
    };
    Ok(Bound::new(cd.py(), COffset::new(typ, cd, ixval))?
        .downcast()?
        .clone())
}

inventory::submit! { CDictionaryRegistryEntry::rust_type::<COffset>("i", &create_index_offset) }
