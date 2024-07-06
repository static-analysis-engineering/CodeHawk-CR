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
use std::collections::BTreeMap;

use itertools::Itertools;
use pyo3::{
    exceptions::{PyException, PyValueError},
    intern,
    prelude::*,
    types::PyFunction,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "indexed_table")?;
    module.add_class::<IndexedTable>()?;
    module.add_class::<IndexedTableValue>()?;
    module.add_function(wrap_pyfunction!(get_key, &module)?)?;
    module.add_function(wrap_pyfunction!(get_rep, &module)?)?;
    module.add_function(wrap_pyfunction!(get_value, &module)?)?;
    Ok(module)
}

pyo3::import_exception!(chc.util.IndexedTable, IndexedTableError);
pyo3::import_exception!(chc.util.IndexedTable, IndexedTableValueMismatchError);

#[pyfunction]
fn get_rep(node: &Bound<PyAny>, /* ET.Element */) -> PyResult<(isize, Vec<String>, Vec<isize>)> {
    let tags = node.call_method1(intern!(node.py(), "get"), (intern!(node.py(), "t"),))?;
    let args = node.call_method1(intern!(node.py(), "get"), (intern!(node.py(), "a"),))?;
    let taglist = if tags.is_none() {
        Vec::new()
    } else {
        tags.extract::<String>()?
            .split(",")
            .map(|x| x.to_string())
            .collect()
    };
    let arglist = if args.is_none() || args.eq("")? {
        Vec::new()
    } else {
        args.extract::<String>()?
            .split(",")
            .map(|x| Ok(x.parse::<isize>()?))
            .collect::<PyResult<Vec<isize>>>()?
    };
    let index_str = node.call_method1(intern!(node.py(), "get"), (intern!(node.py(), "ix"),))?;
    if index_str.is_none() {
        return Err(PyException::new_err(format!(
            "node {} did not have an ix element",
            node
        )));
    }
    let index = index_str.extract::<String>()?.parse::<isize>()?;
    Ok((index, taglist, arglist))
}

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

    pub fn index(&self) -> isize {
        self.index
    }
}

#[pyfunction]
fn get_value<'a, 'b>(
    node: &'a Bound<'b, PyAny>, /* ET.Element */
) -> PyResult<Bound<'b, IndexedTableValue>> {
    let rep = get_rep(node)?;
    Bound::new(node.py(), IndexedTableValue::new(rep.0, rep.1, rep.2))
}

fn element_tree_element<'a, 'py>(py: Python<'py>, tag: &'a str) -> PyResult<Bound<'py, PyAny>> {
    let module = PyModule::import_bound(py, "xml.etree.ElementTree")?;
    module.getattr("Element")?.call1((tag,))
}

/// Table to provide unique indices to objects represented by a key string.
///
/// The table can be checkpointed and reset to that checkpoint with
/// - set_checkpoint
/// - reset_to_checkpoint
///
/// Note: the string encodings use the comma as a concatenation character, hence
///       the comma character cannot be used in any string representation.
#[derive(Clone)]
#[pyclass(subclass)]
pub struct IndexedTable {
    #[pyo3(get)]
    name: String,
    keytable: BTreeMap<(String, String), isize>,
    indextable: BTreeMap<isize, Py<IndexedTableValue>>,
    next: isize,
    reserved: Vec<isize>,
    checkpoint: Option<isize>,
}

#[pymethods]
impl IndexedTable {
    #[new]
    pub fn new(name: String) -> Self {
        IndexedTable {
            name,
            keytable: BTreeMap::new(),
            indextable: BTreeMap::new(),
            next: 1,
            reserved: Vec::new(),
            checkpoint: None,
        }
    }

    fn size(&self) -> isize {
        self.next - 1
    }

    fn reset(&mut self) -> PyResult<()> {
        self.keytable.clear();
        self.indextable.clear();
        self.next = 1;
        self.reserved.clear();
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

    fn iter(&self, f: &Bound<PyFunction>) -> PyResult<()> {
        self.indextable
            .iter()
            .map(|(k, v)| f.call1((*k, v)))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(())
    }

    // Unvalidated
    /// Remove all entries added since the checkpoint was set.
    fn reset_to_checkpoint(&mut self) -> PyResult<isize> {
        let cp = self.checkpoint;
        let Some(cp) = cp else {
            return Err(PyValueError::new_err(
                "Cannot reset non-existent checkpoint",
            ));
        };
        self.indextable.retain(|k, _| {
            if !(cp <= *k && *k < self.next) {
                return true;
            }
            self.reserved.contains(k)
        });
        self.keytable.retain(|_, v| *v < cp);
        self.reserved.clear();
        self.checkpoint.take();
        self.next = cp;
        Ok(cp)
    }

    fn remove_checkpoint(&mut self) {
        self.checkpoint.take();
    }

    // Unvalidated
    fn add(&mut self, key: (String, String), f: &Bound<PyFunction>) -> PyResult<isize> {
        if let Some(value) = self.keytable.get(&key) {
            return Ok(*value);
        }
        let index = self.next;
        let obj = f
            .call1((index, key.clone()))?
            .downcast_into::<IndexedTableValue>()?;
        self.keytable.insert(key, index);
        self.indextable.insert(index, obj.unbind());
        self.next += 1;
        Ok(index)
    }

    fn add_tags_args(
        &mut self,
        tags: Vec<String>,
        args: Vec<isize>,
        f: Bound<PyFunction>,
    ) -> PyResult<isize> {
        let key = get_key(tags.clone(), args.clone());
        if let Some(item) = self.keytable.get(&key) {
            Ok(*item)
        } else {
            let index = self.next;
            let obj = f.call1((index, tags, args))?.downcast()?.clone();
            self.keytable.insert(key, index);
            self.indextable.insert(index, obj.unbind());
            self.next += 1;
            Ok(index)
        }
    }

    fn reserve(&mut self) -> isize {
        let index = self.next;
        self.reserved.push(index);
        self.next += 1;
        index
    }

    fn values(&self) -> Vec<Py<IndexedTableValue>> {
        self.indextable.values().cloned().collect()
    }

    fn items(&self) -> Vec<(isize, Py<IndexedTableValue>)> {
        self.indextable
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    fn commit_reserved(
        &mut self,
        index: isize,
        key: (String, String),
        obj: &Bound<IndexedTableValue>,
    ) -> PyResult<()> {
        if !self.reserved.contains(&index) {
            return Err(IndexedTableError::new_err(format!(
                "Trying to commit nonexisting index: {index}"
            )));
        }
        self.keytable.insert(key, index);
        self.indextable.insert(index, obj.clone().unbind());
        self.reserved.remove(
            self.reserved
                .iter()
                .position(|x| *x == index)
                .ok_or_else(|| PyException::new_err("{index} not in `reserved`"))?,
        );
        Ok(())
    }

    pub fn retrieve(&self, index: isize) -> PyResult<Py<IndexedTableValue>> {
        if let Some(item) = self.indextable.get(&index) {
            return Ok(item.clone());
        }
        let message = format!(
            "Unable to retrieve item {} from table {} (size {})",
            index,
            self.name,
            self.size()
        );
        Err(IndexedTableError::new_err(message))
    }

    fn retrieve_by_key(
        &self,
        f: &Bound<PyFunction>,
    ) -> PyResult<Vec<((String, String), Py<IndexedTableValue>)>> {
        let mut result = Vec::new();
        for (key, index) in self.keytable.iter() {
            if f.call1((key.clone(),))?.extract()? {
                result.push((
                    key.clone(),
                    self.indextable
                        .get(&index)
                        .ok_or_else(|| PyException::new_err("No element at {index}"))?
                        .clone(),
                ));
            }
        }
        Ok(result)
    }

    #[pyo3(signature = (node, f, tag=None))]
    fn write_xml(
        &self,
        node: Bound<PyAny>, // ET.Element
        f: Bound<PyFunction>,
        tag: Option<&str>,
    ) -> PyResult<()> {
        let tag = tag.unwrap_or("n");
        for value in self.indextable.values() {
            let snode = element_tree_element(node.py(), tag)?;
            f.call1((&snode, value))?;
            node.call_method1("append", (snode,))?;
        }
        Ok(())
    }

    #[pyo3(signature = (node, tag, get_value=None, get_key=None, get_index=None))]
    fn read_xml(
        &mut self,
        node: &Bound<PyAny>, // Optional[ET.Element]
        tag: String,
        get_value: Option<Bound<PyFunction>>, // ET.Element -> IndexedTableValue
        get_key: Option<Bound<PyFunction>>,   // IndexedTableValue -> (String, String)
        get_index: Option<Bound<PyFunction>>, // IndexedTableValue -> int
    ) -> PyResult<()> {
        if node.is_none() {
            return Err(IndexedTableError::new_err(format!(
                "Xml node not present in {}",
                self.name
            )));
        }
        for snode in node
            .call_method1(intern!(node.py(), "findall"), (tag,))?
            .extract::<Vec<Bound<PyAny>>>()?
        {
            let obj = if let Some(get_value) = get_value.as_ref() {
                get_value.call1((snode,))?
            } else {
                self::get_value(&snode)?.downcast()?.clone()
            };
            let key: (String, String) = if let Some(get_key) = get_key.as_ref() {
                get_key.call1((obj.clone(),))?
            } else {
                obj.getattr(intern!(node.py(), "key"))?
            }
            .extract()?;
            let index: isize = if let Some(get_index) = get_index.as_ref() {
                get_index.call1((obj.clone(),))?
            } else {
                obj.getattr(intern!(node.py(), "index"))?
            }
            .extract()?;
            self.keytable.insert(key, index);
            self.indextable
                .insert(index, obj.downcast()?.clone().unbind());
            if index >= self.next {
                self.next = index + 1
            }
        }
        Ok(())
    }

    fn objectmap<'a, 'b>(
        &self,
        p: &'a Bound<'b, PyAny>, // int -> IndexedTableValue, but sometimes non-PyFunction
    ) -> PyResult<BTreeMap<isize, Bound<'b, IndexedTableValue>>> {
        self.indextable
            .keys()
            .map(|ix| Ok((*ix, p.call1((*ix,))?.downcast()?.clone())))
            .collect()
    }

    // Unvalidated
    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        let slf_borrow = slf.borrow();
        let mut lines = Vec::new();
        lines.push(format!("\n{}", slf_borrow.name));
        for (ix, obj) in slf_borrow.indextable.iter() {
            lines.push(format!("{ix:>4} {}", obj.bind(slf.py()).str()?));
        }
        if !slf_borrow.reserved.is_empty() {
            lines.push(format!("Reserved: {:?}", slf_borrow.reserved));
        }
        if let Some(cp) = slf_borrow.checkpoint {
            lines.push(format!("Checkpoint: {cp}"));
        }
        Ok(lines.join("\n"))
    }
}

impl IndexedTable {
    pub fn retrieve_bound<'a>(
        &self,
        py: Python<'a>,
        index: isize,
    ) -> PyResult<Bound<'a, IndexedTableValue>> {
        self.retrieve(index).map(|i| i.bind(py).clone())
    }

    pub fn keys(&self) -> impl Iterator<Item = &isize> {
        self.indextable.keys()
    }
}
