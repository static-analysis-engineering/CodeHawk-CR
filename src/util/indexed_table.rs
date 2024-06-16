/*
------------------------------------------------------------------------------
CodeHawk C Analyzer
Author: Henny Sipma
------------------------------------------------------------------------------
The MIT License (MIT)

Copyright (c) 2017-2020 Kestrel Technology LLC
Copyright (c) 2020-2022 Henny B. Sipma
Copyright (c) 2023-2024 Aarno Labs LLC

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
------------------------------------------------------------------------------
*/
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
    fn add_tags_args<'a, 'b>(
        slf: &'a Bound<'b, Self>,
        tags: Vec<String>,
        args: Vec<isize>,
        f: Bound<'b, PyFunction>,
    ) -> PyResult<isize> {
        let mut slf_borrow = slf.borrow_mut();
        let keytable = slf_borrow.keytable.bind_borrowed(slf.py());
        let key = get_key(tags.clone(), args.clone());
        if let Some(item) = keytable.get_item(&key)? {
            Ok(item.extract()?)
        } else {
            let index = slf_borrow.next;
            let obj = f.call1((index, tags, args))?;
            keytable.set_item(&key, index)?;
            let indextable = slf_borrow.indextable.bind_borrowed(slf.py());
            indextable.set_item(&index, obj)?;
            slf_borrow.next += 1;
            Ok(index)
        }
    }

    fn values<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Vec<Bound<'b, IndexedTableValue>>> {
        let slf_borrow = slf.borrow();
        let indextable = slf_borrow.indextable.bind_borrowed(slf.py());
        let mut indexes: Vec<isize> = indextable.keys().extract()?;
        indexes.sort();
        indexes
            .into_iter()
            .map(|k| indextable.get_item(k).map(|v| v.unwrap()))
            .map(|v| Ok(v?.downcast()?.clone()))
            .collect()
    }

    fn retrieve<'a, 'b>(
        slf: &'a Bound<'b, Self>,
        index: isize,
    ) -> PyResult<Bound<'b, IndexedTableValue>> {
        let slf_borrow = slf.borrow();
        let indextable = slf_borrow.indextable.bind_borrowed(slf.py());
        if let Some(item) = indextable.get_item(index)? {
            Ok(item.downcast()?.clone())
        } else {
            let message = format!(
                "Unable to retrieve item {} from table {} (size {})",
                index,
                slf_borrow.name,
                slf_borrow.size()
            );
            Err(IndexedTableError::new_err(message))
        }
    }

    fn write_xml(
        &self,
        py: Python,
        node: Bound<PyAny>, // ET.Element
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
