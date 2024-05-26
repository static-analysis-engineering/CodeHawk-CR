use pyo3::{
    exceptions::PyException,
    prelude::*,
    types::{PyDict, PyList, PyString},
};

#[derive(Clone)]
#[pyclass(subclass)]
pub struct IndexedTableSuperclass {
    #[pyo3(get)]
    name: Py<PyString>,
    #[pyo3(get)]
    keytable: Py<PyDict>,
    #[pyo3(get)]
    indextable: Py<PyDict>,
    #[pyo3(get, set)]
    next: usize,
    #[pyo3(get)]
    reserved: Py<PyList>,
    #[pyo3(get, set)]
    checkpoint: Option<usize>,
}

#[pymethods]
impl IndexedTableSuperclass {
    #[new]
    fn new(py: Python, name: Py<PyString>) -> Self {
        IndexedTableSuperclass {
            name,
            keytable: PyDict::new_bound(py).into(),
            indextable: PyDict::new_bound(py).into(),
            next: 1,
            reserved: PyList::empty_bound(py).into(),
            checkpoint: None,
        }
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
