use pyo3::prelude::*;

mod c_dictionary_record;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "app")?;
    module.add_submodule(&c_dictionary_record::module(py)?)?;
    Ok(module)
}