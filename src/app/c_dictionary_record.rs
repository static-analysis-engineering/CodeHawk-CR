use pyo3::prelude::*;

use crate::util::indexed_table::IndexedTableValue;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_dictionary_record")?;
    module.add_class::<CDictionaryRecord>()?;
    Ok(module)
}

/// Base class for all objects kept in the CDictionary
#[derive(Clone)]
#[pyclass(extends = IndexedTableValue, subclass)]
pub struct CDictionaryRecord {
    #[pyo3(get)]
    cd: Py<PyAny>,
}

#[pymethods]
impl CDictionaryRecord {
    #[new]
    fn new(cd: Py<PyAny>, ixval: IndexedTableValue) -> (CDictionaryRecord, IndexedTableValue) {
        (CDictionaryRecord { cd }, ixval)
    }

    #[getter]
    fn decls(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.cd.getattr(py, "decls")
    }
}
