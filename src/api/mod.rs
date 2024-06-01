use pyo3::prelude::*;

mod api_assumption;
mod interface_dictionary_record;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "api")?;
    module.add_submodule(&api_assumption::module(py)?)?;
    module.add_submodule(&interface_dictionary_record::module(py)?)?;
    Ok(module)
}
