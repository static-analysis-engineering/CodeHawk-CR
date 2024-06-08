use pyo3::prelude::*;

mod c_application;
mod c_attributes;
mod c_const;
mod c_context;
mod c_context_dictionary;
mod c_dictionary;
mod c_dictionary_record;
mod c_file;
mod c_function;
mod c_typ;
mod index_manager;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "app")?;
    module.add_submodule(&c_application::module(py)?)?;
    module.add_submodule(&c_attributes::module(py)?)?;
    module.add_submodule(&c_const::module(py)?)?;
    module.add_submodule(&c_context::module(py)?)?;
    module.add_submodule(&c_context_dictionary::module(py)?)?;
    module.add_submodule(&c_dictionary::module(py)?)?;
    module.add_submodule(&c_dictionary_record::module(py)?)?;
    module.add_submodule(&c_file::module(py)?)?;
    module.add_submodule(&c_function::module(py)?)?;
    module.add_submodule(&c_typ::module(py)?)?;
    module.add_submodule(&index_manager::module(py)?)?;
    Ok(module)
}
