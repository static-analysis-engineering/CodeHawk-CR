mod api;
mod app;
mod cmdline;
mod proof;
mod source;
mod util;

use pyo3::prelude::*;

#[pymodule]
fn chc_rust(py: Python, module: &Bound<PyModule>) -> PyResult<()> {
    module.add_submodule(&api::module(py)?)?;
    module.add_submodule(&app::module(py)?)?;
    module.add_submodule(&cmdline::module(py)?)?;
    module.add_submodule(&proof::module(py)?)?;
    module.add_submodule(&source::module(py)?)?;
    module.add_submodule(&util::module(py)?)?;
    Ok(())
}
