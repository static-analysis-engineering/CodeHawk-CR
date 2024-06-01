use pyo3::prelude::*;

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
#[pyclass(subclass)]
pub struct CDictionary {}

#[pymethods]
impl CDictionary {
    #[new]
    fn new() -> CDictionary {
        CDictionary {}
    }
}
