mod api;
mod app;
mod cmdline;
mod util;

use pyo3::prelude::*;

#[pymodule]
fn chc_rust(py: Python, module: &Bound<PyModule>) -> PyResult<()> {
    module.add_submodule(&api::module(py)?)?;
    module.add_submodule(&app::module(py)?)?;
    module.add_submodule(&cmdline::module(py)?)?;
    module.add_submodule(&util::module(py)?)?;
    Ok(())
}
