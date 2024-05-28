use itertools::Itertools;
use pyo3::{
    intern,
    prelude::*,
    types::{PyDict, PyFunction, PyList, PyString},
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "IndexedTable")?;
    module.add_class::<IndexedTableSuperclass>()?;
    module.add_class::<IndexedTableValue>()?;
    module.add_function(wrap_pyfunction!(get_key, &module)?)?;
    Ok(module)
}

pyo3::import_exception!(chc.util.IndexedTable, IndexedTableError);
pyo3::import_exception!(chc.util.IndexedTable, IndexedTableValueMismatchError);

// TODO: use python types to avoid allocating a String for every tag
#[pyfunction]
fn get_key(tags: Vec<String>, args: Vec<isize>) -> (String, String) {
    (tags.iter().join(","), args.iter().join(","))
}

#[derive(Clone)]
#[pyclass(frozen, subclass)]
pub struct IndexedTableValue {
    #[pyo3(get)]
    index: isize,
    #[pyo3(get)]
    tags: Vec<String>,
    #[pyo3(get)]
    args: Vec<isize>,
}

#[pymethods]
impl IndexedTableValue {
    #[new]
    fn new(index: isize, tags: Vec<String>, args: Vec<isize>) -> IndexedTableValue {
        IndexedTableValue { index, tags, args }
    }

    #[getter]
    fn key(&self) -> (String, String) {
        get_key(self.tags.clone(), self.args.clone())
    }

    fn check_key(&self, reqtagcount: usize, reqargcount: usize, name: String) -> PyResult<()> {
        if self.tags.len() == reqtagcount && self.args.len() == reqargcount {
            Ok(())
        } else {
            let args = (
                self.tags[0].clone(),
                reqtagcount,
                reqargcount,
                self.tags.len(),
                self.args.len(),
                name,
            );
            Err(IndexedTableValueMismatchError::new_err(args))
        }
    }

    fn write_xml(&self, py: Python, node: &Bound<PyAny>) -> PyResult<()> {
        let (tagstr, argstr) = self.key();
        let set = node.getattr(intern!(py, "set"))?;
        if tagstr.len() > 0 {
            set.call1((intern!(py, "t"), tagstr))?;
        }
        if argstr.len() > 0 {
            set.call1((intern!(py, "a"), argstr))?;
        }
        set.call1((intern!(py, "ix"), self.index.to_string()))?;
        Ok(())
    }

    #[pyo3(name = "__str__")]
    fn str(&self) -> String {
        format!(
            concat!(
                "\nIndex table value\n--------------------------\n",
                "index: {}\n",
                "tags: [{}]\n",
                "args: [{}]\n\n"
            ),
            self.index,
            self.tags.join(","),
            self.args.iter().join(",")
        )
    }
}

impl IndexedTableValue {
    pub fn tags(&self) -> &[String] {
        &self.tags[..]
    }

    pub fn args(&self) -> &[isize] {
        &self.args[..]
    }
}

fn element_tree_element<'a, 'py>(py: Python<'py>, tag: &'a str) -> PyResult<Bound<'py, PyAny>> {
    let module = PyModule::import_bound(py, "xml.etree.ElementTree")?;
    let tag_pystr = PyString::new_bound(py, tag);
    module.getattr("Element")?.call1((tag_pystr,))
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
    next: isize,
    #[pyo3(get)]
    reserved: Py<PyList>,
    #[pyo3(get)]
    checkpoint: Option<isize>,
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

    fn size(&self) -> isize {
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

    fn set_checkpoint(&mut self) -> PyResult<isize> {
        if let Some(n) = self.checkpoint {
            let message = format!("Checkpoint has already been set at {n}");
            Err(IndexedTableError::new_err(message))
        } else {
            self.checkpoint = Some(self.next);
            Ok(self.next)
        }
    }

    fn remove_checkpoint(&mut self) {
        self.checkpoint.take();
    }

    // TODO: use python types to avoid allocating a String for every tag
    fn add_tags_args<'a, 'py>(
        &'a mut self,
        py: Python<'py>,
        tags: Vec<String>,
        args: Vec<isize>,
        f: Bound<'py, PyFunction>,
    ) -> PyResult<isize> {
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

    fn values<'a, 'py>(&'a self, py: Python<'py>) -> PyResult<Vec<Bound<'py, PyAny>>> {
        let indextable = self.indextable.bind_borrowed(py);
        let mut indexes: Vec<isize> = indextable.keys().extract()?;
        indexes.sort();
        indexes
            .into_iter()
            .map(|k| indextable.get_item(k).map(|v| v.unwrap()))
            .collect()
    }

    fn retrieve<'a, 'py>(&'a self, py: Python<'py>, index: isize) -> PyResult<Bound<'py, PyAny>> {
        if let Some(item) = self.indextable.bind_borrowed(py).get_item(index)? {
            Ok(item)
        } else {
            let message = format!(
                "Unable to retrieve item {} from table {} (size {})",
                index,
                self.name,
                self.size()
            );
            Err(IndexedTableError::new_err(message))
        }
    }

    fn write_xml(
        &self,
        py: Python,
        node: Bound<PyAny>,
        f: Bound<PyFunction>,
        tag: Option<&str>,
    ) -> PyResult<()> {
        let tag = tag.unwrap_or("n");
        let indextable = self.indextable.bind_borrowed(py);
        let mut indexes: Vec<isize> = indextable.keys().extract()?;
        indexes.sort();
        for key in indexes {
            let snode = element_tree_element(py, tag)?;
            f.call1((&snode, indextable.get_item(key)?.unwrap()))?;
            node.call_method1("append", (snode,))?;
        }
        Ok(())
    }
}
