use pyo3::prelude::*;

mod c_application;
mod c_const;
mod c_context;
mod c_dictionary;
mod c_dictionary_record;
mod c_file;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "app")?;
    module.add_submodule(&c_application::module(py)?)?;
    module.add_submodule(&c_const::module(py)?)?;
    module.add_submodule(&c_context::module(py)?)?;
    module.add_submodule(&c_dictionary::module(py)?)?;
    module.add_submodule(&c_dictionary_record::module(py)?)?;
    module.add_submodule(&c_file::module(py)?)?;
    Ok(module)
}
