use pyo3::{intern, prelude::*};

use crate::util::indexed_table::IndexedTableValue;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_context")?;
    module.add_class::<CContextDictionaryRecord>()?;
    Ok(module)
}

#[derive(Clone)]
#[pyclass(extends = IndexedTableValue, frozen, subclass)]
pub struct CContextDictionaryRecord {
    #[pyo3(get)]
    cxd: Py<PyAny>,
}

#[pymethods]
impl CContextDictionaryRecord {
    #[new]
    pub fn new(
        cxd: Py<PyAny>,
        ixval: IndexedTableValue,
    ) -> (CContextDictionaryRecord, IndexedTableValue) {
        (CContextDictionaryRecord { cxd }, ixval)
    }

    #[pyo3(name = "__str__")]
    pub fn str(slf: Py<Self>, py: Python) -> PyResult<String> {
        Ok(format!(
            "context-record: {}",
            slf.getattr(py, intern!(py, "key"))?
        ))
    }
}
