use pyo3::prelude::*;

pub mod c_function_proofs;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "proof")?;
    module.add_submodule(&c_function_proofs::module(py)?)?;
    Ok(module)
}
