use pyo3::prelude::*;

pub mod c_fun_inv_dictionary;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "invariants")?;
    module.add_submodule(&c_fun_inv_dictionary::module(py)?)?;
    Ok(module)
}
