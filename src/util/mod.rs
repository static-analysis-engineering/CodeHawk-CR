use pyo3::prelude::*;

pub mod indexed_table;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "util")?;
    module.add_submodule(&indexed_table::module(py)?)?;
    Ok(module)
}
