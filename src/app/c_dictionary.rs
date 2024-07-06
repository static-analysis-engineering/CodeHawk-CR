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
use std::collections::BTreeMap;

use pyo3::{prelude::*, type_object::PyTypeInfo, types::PyType};

use crate::{
    app::{
        c_attributes::{CAttr, CAttribute, CAttributes},
        c_const::CConst,
        c_dictionary_record::{cdregistry, CDictionaryRecordTrait},
    },
    util::indexed_table::IndexedTable,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_dictionary")?;
    module.add_class::<CDictionary>()?;
    Ok(module)
}

/// Indexed types.
///
/// subclassed by
///
/// - CFileDictionary: Corresponds with cchlib/cCHDictionary.
/// - CGlobalDictionary: constructed in the python api
#[derive(Clone)]
#[pyclass(get_all, subclass)]
pub struct CDictionary {
    attrparam_table: Py<IndexedTable>,
    attribute_table: Py<IndexedTable>,
    attributes_table: Py<IndexedTable>,
    constant_table: Py<IndexedTable>,
    exp_table: Py<IndexedTable>,
    funarg_table: Py<IndexedTable>,
    funargs_table: Py<IndexedTable>,
    lhost_table: Py<IndexedTable>,
    lval_table: Py<IndexedTable>,
    offset_table: Py<IndexedTable>,
    typ_table: Py<IndexedTable>,
    typsig_table: Py<IndexedTable>,
    typsiglist_table: Py<IndexedTable>,
}

#[pymethods]
impl CDictionary {
    #[new]
    fn new(py: Python) -> PyResult<CDictionary> {
        Ok(CDictionary {
            attrparam_table: Py::new(py, IndexedTable::new("attrparam-table".to_string()))?,
            attribute_table: Py::new(py, IndexedTable::new("attribute-table".to_string()))?,
            attributes_table: Py::new(py, IndexedTable::new("attributes-table".to_string()))?,
            constant_table: Py::new(py, IndexedTable::new("constant-table".to_string()))?,
            exp_table: Py::new(py, IndexedTable::new("exp-table".to_string()))?,
            funarg_table: Py::new(py, IndexedTable::new("funarg-table".to_string()))?,
            funargs_table: Py::new(py, IndexedTable::new("funargs-table".to_string()))?,
            lhost_table: Py::new(py, IndexedTable::new("lhost-table".to_string()))?,
            lval_table: Py::new(py, IndexedTable::new("lval-table".to_string()))?,
            offset_table: Py::new(py, IndexedTable::new("offset-table".to_string()))?,
            typ_table: Py::new(py, IndexedTable::new("typ-table".to_string()))?,
            typsig_table: Py::new(py, IndexedTable::new("typsig-table".to_string()))?,
            typsiglist_table: Py::new(py, IndexedTable::new("typsiglist-table".to_string()))?,
        })
    }

    #[getter]
    fn tables(&self) -> Vec<Py<IndexedTable>> {
        vec![
            self.attrparam_table.clone(),
            self.attribute_table.clone(),
            self.attributes_table.clone(),
            self.constant_table.clone(),
            self.exp_table.clone(),
            self.funarg_table.clone(),
            self.funargs_table.clone(),
            self.lhost_table.clone(),
            self.lval_table.clone(),
            self.offset_table.clone(),
            self.typ_table.clone(),
            self.typsig_table.clone(),
            self.typsiglist_table.clone(),
        ]
    }

    // -------------- Retrieve items from dictionary tables -------------------

    pub fn get_attrparam<'a>(slf: &Bound<'a, Self>, ix: isize) -> PyResult<Bound<'a, CAttr>> {
        Self::dict_to_registry(slf, &slf.borrow().attrparam_table, ix)
    }

    fn get_attrparam_map<'a>(slf: &Bound<'a, Self>) -> PyResult<BTreeMap<isize, Bound<'a, CAttr>>> {
        Self::object_map(slf, &slf.borrow().attrparam_table, Self::get_attrparam)
    }

    pub fn get_attribute<'a>(slf: &Bound<'a, Self>, ix: isize) -> PyResult<Bound<'a, CAttribute>> {
        Self::dict_to_constructor(slf, &slf.borrow().attribute_table, ix)
    }

    fn get_attribute_map<'a>(
        slf: &Bound<'a, Self>,
    ) -> PyResult<BTreeMap<isize, Bound<'a, CAttribute>>> {
        Self::object_map(slf, &slf.borrow().attribute_table, Self::get_attribute)
    }

    pub fn get_attributes<'a>(
        slf: &Bound<'a, Self>,
        ix: isize,
    ) -> PyResult<Bound<'a, CAttributes>> {
        Self::dict_to_constructor(slf, &slf.borrow().attributes_table, ix)
    }

    fn get_attributes_map<'a>(
        slf: &Bound<'a, Self>,
    ) -> PyResult<BTreeMap<isize, Bound<'a, CAttributes>>> {
        Self::object_map(slf, &slf.borrow().attributes_table, Self::get_attributes)
    }

    fn get_constant<'a>(slf: &Bound<'a, Self>, ix: isize) -> PyResult<Bound<'a, CConst>> {
        Self::dict_to_registry(slf, &slf.borrow().constant_table, ix)
    }

    fn get_constant_map<'a>(slf: &Bound<'a, Self>) -> PyResult<BTreeMap<isize, Bound<'a, CConst>>> {
        Self::object_map(slf, &slf.borrow().constant_table, Self::get_constant)
    }
}

impl CDictionary {
    fn dict_to_registry<'a, T: PyTypeInfo>(
        slf: &Bound<'a, Self>,
        dict: &Py<IndexedTable>,
        ix: isize,
    ) -> PyResult<Bound<'a, T>> {
        let py = slf.py();
        let ixval = dict.borrow(py).retrieve_bound(py, ix)?;
        Ok(cdregistry(py)?
            .mk_instance(slf, &ixval, &PyType::new_bound::<T>(py))?
            .downcast()?
            .clone())
    }

    fn dict_to_constructor<'a, T: CDictionaryRecordTrait>(
        slf: &Bound<'a, Self>,
        dict: &Py<IndexedTable>,
        ix: isize,
    ) -> PyResult<Bound<'a, T>> {
        let py = slf.py();
        let ixval = dict.borrow(py).retrieve_bound(py, ix)?;
        Bound::new(
            slf.py(),
            T::new(slf.clone().unbind(), ixval.borrow().clone()),
        )
    }

    fn object_map<'a, T, F>(
        slf: &Bound<'a, Self>,
        dict: &Py<IndexedTable>,
        func: F,
    ) -> PyResult<BTreeMap<isize, Bound<'a, T>>>
    where
        F: Fn(&Bound<'a, Self>, isize) -> PyResult<Bound<'a, T>>,
    {
        dict.borrow(slf.py())
            .keys()
            .map(|k| Ok((*k, func(slf, *k)?)))
            .collect()
    }
}
