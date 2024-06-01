use pyo3::prelude::*;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_file")?;
    module.add_class::<CFile>()?;
    Ok(module)
}

/// C File main access point.
#[derive(Clone)]
#[pyclass(subclass)]
pub struct CFile {}

#[pymethods]
impl CFile {
    #[new]
    fn new() -> CFile {
        CFile {}
    }
}
