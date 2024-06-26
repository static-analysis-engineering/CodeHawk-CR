use pyo3::prelude::*;

mod test_c_file_ref;
mod test_c_function_ref;
mod test_manager;
mod test_ppo_ref;
mod test_results;
mod test_set_ref;
mod test_spo_ref;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "kendra")?;
    module.add_submodule(&test_c_file_ref::module(py)?)?;
    module.add_submodule(&test_c_function_ref::module(py)?)?;
    module.add_submodule(&test_manager::module(py)?)?;
    module.add_submodule(&test_ppo_ref::module(py)?)?;
    module.add_submodule(&test_results::module(py)?)?;
    module.add_submodule(&test_spo_ref::module(py)?)?;
    module.add_submodule(&test_set_ref::module(py)?)?;
    Ok(module)
}
