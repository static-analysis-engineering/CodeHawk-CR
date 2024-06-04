use pyo3::prelude::*;

mod analysis_manager;
mod kendra;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "cmdline")?;
    module.add_submodule(&analysis_manager::module(py)?)?;
    module.add_submodule(&kendra::module(py)?)?;
    Ok(module)
}
