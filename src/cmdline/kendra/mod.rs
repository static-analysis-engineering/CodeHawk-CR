use pyo3::prelude::*;

mod test_set_ref;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "kendra")?;
    module.add_submodule(&test_set_ref::module(py)?)?;
    Ok(module)
}
