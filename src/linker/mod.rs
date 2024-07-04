use pyo3::prelude::*;

pub mod c_linker;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "linker")?;
    module.add_submodule(&c_linker::module(py)?)?;
    Ok(module)
}
