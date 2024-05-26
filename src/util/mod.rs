use pyo3::prelude::*;

mod indexed_table;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "util")?;
    module.add_class::<indexed_table::IndexedTableSuperclass>()?;
    Ok(module)
}
