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
use once_cell::sync::OnceCell;
use pyo3::{
    intern,
    prelude::*,
    types::{PyDict, PyType},
};

use crate::{
    app::{c_declarations::CDeclarations, c_dictionary::CDictionary},
    util::indexed_table::IndexedTableValue,
};

pyo3::import_exception!(chc.util.fileutil, CHCError);

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_dictionary_record")?;
    module.add_class::<CDictionaryRecord>()?;
    module.add_class::<CDictionaryRegistry>()?;
    module.add("cdregistry", cdregistry(py)?)?;
    module.add_class::<CDeclarationsRecord>()?;
    Ok(module)
}

/// Base class for all objects kept in the CDictionary
#[derive(Clone)]
#[pyclass(extends = IndexedTableValue, frozen, subclass)]
pub struct CDictionaryRecord {
    #[pyo3(get)]
    cd: Py<CDictionary>,
}

#[pymethods]
impl CDictionaryRecord {
    #[new]
    pub fn new(
        cd: Py<CDictionary>,
        ixval: IndexedTableValue,
    ) -> (CDictionaryRecord, IndexedTableValue) {
        (CDictionaryRecord { cd }, ixval)
    }

    #[getter]
    pub fn decls(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.cd.getattr(py, intern!(py, "decls"))
    }
}

impl CDictionaryRecord {
    pub fn cd(&self) -> Py<CDictionary> {
        self.cd.clone()
    }
}

/// Base class for all objects kept in the CFileDeclarations.
#[derive(Clone)]
#[pyclass(extends = IndexedTableValue, frozen, subclass)]
pub struct CDeclarationsRecord {
    #[pyo3(get)]
    decls: Py<CDeclarations>,
}

#[pymethods]
impl CDeclarationsRecord {
    #[new]
    pub fn new(
        decls: Py<CDeclarations>,
        ixval: IndexedTableValue,
    ) -> (CDeclarationsRecord, IndexedTableValue) {
        (CDeclarationsRecord { decls }, ixval)
    }

    #[getter]
    pub fn dictionary(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.decls.getattr(py, intern!(py, "dictionary"))
    }
}

impl CDeclarationsRecord {
    pub fn decls(&self) -> Py<CDeclarations> {
        self.decls.clone()
    }
}

#[derive(Clone)]
#[pyclass(frozen)]
pub struct CDictionaryRegistryHandler {
    registry: Py<CDictionaryRegistry>,
    tag: String,
    anchor: Py<PyType>,
}

#[pymethods]
impl CDictionaryRegistryHandler {
    #[pyo3(name = "__call__")]
    fn call(slf: &Bound<Self>, t: Py<PyType>) -> PyResult<Py<PyType>> {
        let slf_get = slf.get();
        slf_get
            .registry
            .bind(slf.py())
            .borrow()
            .register
            .bind(slf.py())
            .set_item((slf_get.anchor.clone(), slf_get.tag.as_str()), t.clone())?;
        Ok(t)
    }
}

#[derive(Clone)]
#[pyclass(get_all, subclass)]
pub struct CDictionaryRegistry {
    register: Py<PyDict>,
}

#[pymethods]
impl CDictionaryRegistry {
    #[new]
    pub fn new(py: Python) -> CDictionaryRegistry {
        CDictionaryRegistry {
            register: PyDict::new_bound(py).unbind(),
        }
    }

    fn register_tag(
        registry: Py<Self>,
        tag: String,
        anchor: Py<PyType>,
    ) -> CDictionaryRegistryHandler {
        CDictionaryRegistryHandler {
            registry,
            tag,
            anchor,
        }
    }

    fn mk_instance<'a, 'b, 'c, 'd, 'e>(
        slf: &'b Bound<'a, Self>,
        cd: &'c Bound<'a, CDictionary>,
        ixval: &'d Bound<'a, IndexedTableValue>,
        anchor: &'e Bound<'a, PyType>,
    ) -> PyResult<Bound<'a, CDictionaryRecord>> {
        let tag = ixval.get().tags()[0].clone();
        let Some(item) = slf
            .borrow()
            .register
            .bind(slf.py())
            .get_item((anchor, tag.as_str()))?
        else {
            return Err(CHCError::new_err(format!(
                "Unknown cdictionary type: {tag}"
            )));
        };
        Ok(item.call1((cd, ixval))?.downcast()?.clone())
    }
}

pub struct CDictionaryRegistryEntry {
    pub make: &'static (dyn Sync + Fn(Python) -> (Py<PyType>, &'static str, Py<PyType>)),
}

inventory::collect!(CDictionaryRegistryEntry);

static CDREGISTRY: OnceCell<Py<CDictionaryRegistry>> = OnceCell::new();

pub fn cdregistry(py: Python) -> PyResult<Py<CDictionaryRegistry>> {
    CDREGISTRY
        .get_or_try_init(|| {
            let registry = CDictionaryRegistry::new(py);
            for entry in inventory::iter::<CDictionaryRegistryEntry>() {
                let (anchor, tag, t) = (entry.make)(py);
                registry.register.bind(py).set_item((anchor, tag), t)?;
            }
            Py::new(py, registry)
        })
        .cloned()
}
