use pyo3::prelude::*;

pub mod c_src_file;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "source")?;
    module.add_submodule(&c_src_file::module(py)?)?;
    Ok(module)
}
