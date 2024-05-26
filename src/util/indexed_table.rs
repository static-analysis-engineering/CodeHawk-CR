use pyo3::{exceptions::PyException, prelude::*, types::PyString};

#[derive(Clone)]
#[pyclass(subclass)]
pub struct IndexedTableSuperclass {
    #[pyo3(get)]
    name: Py<PyString>,
}

#[pymethods]
impl IndexedTableSuperclass {
    #[new]
    fn new(name: Py<PyString>) -> Self {
        IndexedTableSuperclass { name }
    }

    fn size(&self) -> PyResult<usize> {
        Err(PyException::new_err(
            "size not overridden in IndexedTableSuperclass",
        ))
    }

    fn reset(&mut self) -> PyResult<()> {
        Err(PyException::new_err(
            "reset not overridden in IndexedTableSuperclass",
        ))
    }
}
