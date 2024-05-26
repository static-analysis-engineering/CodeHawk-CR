use pyo3::{
    prelude::*,
    types::{PyDict, PyList, PyString},
};

fn indexed_table_error(py: Python, message: String) -> PyResult<PyErr> {
    let module = PyModule::import_bound(py, "chc.util.IndexedTable")?;
    let message_pystr = PyString::new_bound(py, &message);
    let value = module
        .getattr("IndexedTableError")?
        .call1((message_pystr,))?;
    Ok(PyErr::from_value_bound(value))
}

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
    #[pyo3(get)]
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

    fn size(&self) -> usize {
        self.next - 1
    }

    fn reset(&mut self, py: Python) -> PyResult<()> {
        self.keytable.bind_borrowed(py).clear();
        self.indextable.bind_borrowed(py).clear();
        self.next = 1;
        self.reserved.bind_borrowed(py).call_method0("clear")?;
        self.checkpoint = None;
        Ok(())
    }

    fn set_checkpoint(&mut self, py: Python) -> PyResult<usize> {
        if let Some(n) = self.checkpoint {
            let message = format!("Checkpoint has already been set at {n}");
            Err(indexed_table_error(py, message)?)
        } else {
            self.checkpoint = Some(self.next);
            Ok(self.next)
        }
    }
}
