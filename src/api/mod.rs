use pyo3::prelude::*;

mod api_assumption;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "api")?;
    module.add_submodule(&api_assumption::module(py)?)?;
    Ok(module)
}
