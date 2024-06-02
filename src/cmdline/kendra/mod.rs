use pyo3::prelude::*;

mod test_c_file_ref;
mod test_c_function_ref;
mod test_manager;
mod test_set_ref;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "kendra")?;
    module.add_submodule(&test_c_file_ref::module(py)?)?;
    module.add_submodule(&test_c_function_ref::module(py)?)?;
    module.add_submodule(&test_manager::module(py)?)?;
    module.add_submodule(&test_set_ref::module(py)?)?;
    Ok(module)
}
