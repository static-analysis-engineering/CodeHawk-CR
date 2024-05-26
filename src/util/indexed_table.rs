use itertools::Itertools;
use pyo3::{
    intern,
    prelude::*,
    types::{PyDict, PyFunction, PyList, PyString},
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "IndexedTable")?;
    module.add_class::<IndexedTableSuperclass>()?;
    module.add_function(wrap_pyfunction!(get_key, &module)?)?;
    Ok(module)
}

// TODO: use python types to avoid allocating a String for every tag
#[pyfunction]
fn get_key(tags: Vec<String>, args: Vec<usize>) -> (String, String) {
    (tags.iter().join(","), args.iter().join(","))
}

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
struct IndexedTableSuperclass {
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
        self.reserved
            .bind_borrowed(py)
            .call_method0(intern!(py, "clear"))?;
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

    // TODO: use python types to avoid allocating a String for every tag
    fn add_tags_args<'a, 'py>(
        &'a mut self,
        py: Python<'py>,
        tags: Vec<String>,
        args: Vec<usize>,
        f: Bound<'py, PyFunction>,
    ) -> PyResult<usize> {
        let key = get_key(tags.clone(), args.clone());
        if let Some(item) = self.keytable.bind_borrowed(py).get_item(&key)? {
            FromPyObject::extract_bound(&item)
        } else {
            let index = self.next;
            let obj = f.call1((index, tags, args))?;
            self.keytable.bind_borrowed(py).set_item(&key, index)?;
            self.indextable.bind_borrowed(py).set_item(&index, obj)?;
            self.next += 1;
            Ok(index)
        }
    }

    fn retrieve<'a, 'py>(&'a self, py: Python<'py>, index: usize) -> PyResult<Bound<'py, PyAny>> {
        if let Some(item) = self.indextable.bind_borrowed(py).get_item(index)? {
            Ok(item)
        } else {
            let message = format!(
                "Unable to retrieve item {} from table {} (size {})",
                index,
                self.name,
                self.size()
            );
            Err(indexed_table_error(py, message)?)
        }
    }
}
