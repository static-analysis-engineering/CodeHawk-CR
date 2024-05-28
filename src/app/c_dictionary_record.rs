use pyo3::{intern, prelude::*};

use crate::util::indexed_table::IndexedTableValue;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_dictionary_record")?;
    module.add_class::<CDictionaryRecord>()?;
    module.add_class::<CDeclarationsRecord>()?;
    Ok(module)
}

/// Base class for all objects kept in the CDictionary
#[derive(Clone)]
#[pyclass(extends = IndexedTableValue, frozen, subclass)]
pub struct CDictionaryRecord {
    #[pyo3(get)]
    cd: Py<PyAny>,
}

#[pymethods]
impl CDictionaryRecord {
    #[new]
    pub fn new(cd: Py<PyAny>, ixval: IndexedTableValue) -> (CDictionaryRecord, IndexedTableValue) {
        (CDictionaryRecord { cd }, ixval)
    }

    #[getter]
    fn decls(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.cd.getattr(py, intern!(py, "decls"))
    }
}

/// Base class for all objects kept in the CFileDeclarations.
#[derive(Clone)]
#[pyclass(extends = IndexedTableValue, frozen, subclass)]
struct CDeclarationsRecord {
    #[pyo3(get)]
    decls: Py<PyAny>,
}

#[pymethods]
impl CDeclarationsRecord {
    #[new]
    fn new(decls: Py<PyAny>, ixval: IndexedTableValue) -> (CDeclarationsRecord, IndexedTableValue) {
        (CDeclarationsRecord { decls }, ixval)
    }

    #[getter]
    fn dictionary(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.decls.getattr(py, intern!(py, "dictionary"))
    }
}
