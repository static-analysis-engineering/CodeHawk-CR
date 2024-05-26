mod util;

use pyo3::prelude::*;

#[pymodule]
fn chc_rust(py: Python, module: &Bound<PyModule>) -> PyResult<()> {
    module.add_submodule(&util::module(py)?)?;
    Ok(())
}
