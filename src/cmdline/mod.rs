use pyo3::prelude::*;

mod kendra;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "cmdline")?;
    module.add_submodule(&kendra::module(py)?)?;
    Ok(module)
}
