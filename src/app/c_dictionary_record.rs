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
    pyclass::PyClass,
    type_object::PyTypeInfo,
    types::{PyCFunction, PyDict, PyTuple, PyType},
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

pub trait CDictionaryRecordTrait: PyClass {
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self>;
}

/// Base class for all objects kept in the CDictionary
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
    pub fn cd(&self) -> &Py<CDictionary> {
        &self.cd
    }
}

/// Base class for all objects kept in the CFileDeclarations.
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

#[pyclass]
pub struct CDictionaryRegistry {
    #[pyo3(get)]
    register: Py<PyDict>,
    // TODO figure out how to use `anchor` as a key
    rust_register: Vec<CDictionaryRegistryEntry>,
}

impl CDictionaryRegistry {
    pub fn new(py: Python, rust_register: Vec<CDictionaryRegistryEntry>) -> CDictionaryRegistry {
        CDictionaryRegistry {
            register: PyDict::new_bound(py).unbind(),
            rust_register,
        }
    }
}

#[pymethods]
impl CDictionaryRegistry {
    fn register_tag<'a>(
        slf_registry: Py<Self>,
        py: Python<'a>,
        tag: String,
        anchor: Py<PyType>,
    ) -> PyResult<Bound<'a, PyCFunction>> {
        let closure =
            move |tuple: &Bound<PyTuple>, _dict: Option<&Bound<PyDict>>| -> PyResult<Py<PyType>> {
                let (t,): (Py<PyType>,) = tuple.extract()?;
                slf_registry
                    .borrow(tuple.py())
                    .register
                    .bind(tuple.py())
                    .set_item((anchor.clone(), tag.as_str()), t.clone())?;
                Ok(t)
            };
        Ok(PyCFunction::new_closure_bound(py, None, None, closure)?)
    }

    pub fn mk_instance<'a, 'b, 'c, 'd, 'e>(
        &self,
        cd: &'c Bound<'a, CDictionary>,
        ixval: &'d Bound<'a, IndexedTableValue>,
        anchor: &'e Bound<'a, PyType>,
    ) -> PyResult<Bound<'a, CDictionaryRecord>> {
        let tag = ixval.get().tags()[0].clone();
        for entry in &self.rust_register {
            if entry.matches(cd.py(), anchor, tag.as_str())? {
                return entry.mk_instance(cd, ixval);
            }
        }
        let Some(item) = self
            .register
            .bind(cd.py())
            .get_item((anchor, tag.as_str()))?
        else {
            return Err(CHCError::new_err(format!(
                "Unknown cdictionary type: {tag}"
            )));
        };
        Ok(item.call1((cd, ixval))?.downcast()?.clone())
    }
}

#[derive(Clone)]
pub enum CDictionaryRegistryEntry {
    PythonType {
        tag: &'static str,
        create: &'static (dyn Sync + Fn(Python) -> (Py<PyType>, Py<PyType>)),
    },
    RustType {
        tag: &'static str,
        anchor: &'static (dyn Sync + Fn(Python) -> Py<PyType>),
        create: &'static (dyn Sync
                      + for<'a, 'b> Fn(
            &'a Bound<'b, CDictionary>,
            IndexedTableValue,
        ) -> PyResult<Bound<'b, CDictionaryRecord>>),
    },
}

fn create_entry_python_types<Anchor: PyTypeInfo + 'static, T: PyTypeInfo + 'static>(
    py: Python,
) -> (Py<PyType>, Py<PyType>) {
    (
        PyType::new_bound::<Anchor>(py).unbind(),
        PyType::new_bound::<T>(py).unbind(),
    )
}

fn create_entry_anchor<Anchor: PyTypeInfo + 'static>(py: Python) -> Py<PyType> {
    PyType::new_bound::<Anchor>(py).unbind()
}

impl CDictionaryRegistryEntry {
    pub const fn python_type<Anchor: PyTypeInfo + 'static, T: PyTypeInfo + 'static>(
        tag: &'static str,
    ) -> CDictionaryRegistryEntry {
        CDictionaryRegistryEntry::PythonType {
            tag,
            create: &create_entry_python_types::<Anchor, T>,
        }
    }

    pub const fn rust_type<Anchor: PyTypeInfo + 'static>(
        tag: &'static str,
        create: &'static (dyn Sync
                      + for<'a, 'b> Fn(
            &'a Bound<'b, CDictionary>,
            IndexedTableValue,
        ) -> PyResult<Bound<'b, CDictionaryRecord>>),
    ) -> CDictionaryRegistryEntry {
        CDictionaryRegistryEntry::RustType {
            tag,
            anchor: &create_entry_anchor::<Anchor>,
            create,
        }
    }

    fn matches(&self, py: Python, anchor_in: &Bound<PyType>, tag_in: &str) -> PyResult<bool> {
        Ok(match self {
            Self::PythonType { tag, create } => {
                *tag == tag_in && create(py).0.bind(py).eq(anchor_in)?
            }
            Self::RustType { tag, anchor, .. } => {
                *tag == tag_in && anchor(py).bind(py).eq(anchor_in)?
            }
        })
    }

    fn mk_instance<'a, 'b, 'c, 'd>(
        &self,
        cd: &'b Bound<'a, CDictionary>,
        ixval: &'c Bound<'a, IndexedTableValue>,
    ) -> PyResult<Bound<'a, CDictionaryRecord>> {
        Ok(match self {
            Self::PythonType { create, .. } => create(cd.py())
                .1
                .bind(cd.py())
                .call1((cd, ixval))?
                .downcast()?
                .clone(),
            Self::RustType { create, .. } => {
                let ixval = ixval.get().clone();
                create(cd, ixval)?
            }
        })
    }
}

inventory::collect!(CDictionaryRegistryEntry);

static CDREGISTRY: OnceCell<Py<CDictionaryRegistry>> = OnceCell::new();

pub fn cdregistry(py: Python) -> PyResult<PyRef<CDictionaryRegistry>> {
    CDREGISTRY
        .get_or_try_init(|| {
            let entries = inventory::iter::<CDictionaryRegistryEntry>()
                .cloned()
                .collect();
            let registry = CDictionaryRegistry::new(py, entries);
            Py::new(py, registry)
        })
        .map(|reg| reg.borrow(py))
}
