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
    types::{PyDict, PyFunction, PyList, PyString},
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
    let tag_pystr = PyString::new_bound(py, tag);
    module.getattr("Element")?.call1((tag_pystr,))
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
struct IndexedTable {
    #[pyo3(get)]
    name: String,
    keytable: Py<PyDict>,   // (str, str) -> int
    indextable: Py<PyDict>, // int -> IndexedTableValue
    next: isize,
    reserved: Py<PyList>, // int list
    checkpoint: Option<isize>,
}

#[pymethods]
impl IndexedTable {
    #[new]
    fn new(py: Python, name: String) -> Self {
        IndexedTable {
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

    fn iter(slf: &Bound<Self>, f: &Bound<PyFunction>) -> PyResult<()> {
        IndexedTable::items(slf)?
            .into_iter()
            .map(|p| f.call1(p))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(())
    }

    // Unvalidated
    /// Remove all entries added since the checkpoint was set.
    fn reset_to_checkpoint(slf: &Bound<Self>) -> PyResult<isize> {
        let mut slf_borrow = slf.borrow_mut();
        let cp = slf_borrow.checkpoint;
        let Some(cp) = cp else {
            return Err(PyValueError::new_err(
                "Cannot reset non-existent checkpoint",
            ));
        };
        let reserved = slf_borrow.reserved.bind(slf.py());
        let indextable = slf_borrow.indextable.bind(slf.py());
        for i in cp..slf_borrow.next {
            if !reserved.contains(i)? {
                indextable.del_item(i)?;
            }
        }
        let keytable = slf_borrow.keytable.bind(slf.py());
        for p in keytable.items() {
            let (k, v): (isize, isize) = p.extract()?;
            if v >= cp {
                keytable.del_item(k)?;
            }
        }
        reserved.call_method0(intern!(slf.py(), "clear"))?;
        slf_borrow.checkpoint.take();
        slf_borrow.next = cp;
        Ok(cp)
    }

    fn remove_checkpoint(&mut self) {
        self.checkpoint.take();
    }

    // Unvalidated
    fn add(slf: &Bound<Self>, key: (String, String), f: &Bound<PyFunction>) -> PyResult<isize> {
        let mut slf_borrow = slf.borrow_mut();
        let keytable = slf_borrow.keytable.bind(slf.py());
        if let Some(value) = keytable.get_item(&key)? {
            return Ok(value.extract()?);
        }
        let index = slf_borrow.next;
        let obj = f
            .call1((index, key.clone()))?
            .downcast_into::<IndexedTableValue>()?;
        keytable.set_item(key, index)?;
        slf_borrow.indextable.bind(slf.py()).set_item(index, obj)?;
        slf_borrow.next += 1;
        Ok(index)
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

    fn reserve(slf: &Bound<Self>) -> PyResult<isize> {
        let mut slf_borrow = slf.borrow_mut();
        let index = slf_borrow.next;
        slf_borrow.reserved.bind(slf.py()).append(index)?;
        slf_borrow.next += 1;
        Ok(index)
    }

    fn values<'a, 'b>(slf: &'a Bound<'b, Self>) -> PyResult<Vec<Bound<'b, IndexedTableValue>>> {
        Ok(IndexedTable::items(slf)?.into_iter().map(|p| p.1).collect())
    }

    fn items<'a, 'b>(
        slf: &'a Bound<'b, Self>,
    ) -> PyResult<Vec<(isize, Bound<'b, IndexedTableValue>)>> {
        let slf_borrow = slf.borrow();
        let indextable = slf_borrow.indextable.bind_borrowed(slf.py());
        let mut elems = indextable
            .iter()
            .map(|(key, value)| Ok((key.extract()?, value.downcast()?.clone())))
            .collect::<PyResult<Vec<(isize, Bound<'b, IndexedTableValue>)>>>()?;
        elems.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(elems)
    }

    fn commit_reserved(
        slf: &Bound<Self>,
        index: isize,
        key: (String, String),
        obj: &Bound<IndexedTableValue>,
    ) -> PyResult<()> {
        let slf_borrow = slf.borrow();
        let reserved = slf_borrow.reserved.bind(slf.py());
        if !reserved.contains(index)? {
            return Err(IndexedTableError::new_err(format!(
                "Trying to commit nonexisting index: {index}"
            )));
        }
        slf_borrow.keytable.bind(slf.py()).set_item(key, index)?;
        slf_borrow.indextable.bind(slf.py()).set_item(index, obj)?;
        reserved.call_method1(intern!(slf.py(), "remove"), (index,))?;
        Ok(())
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

    fn retrieve_by_key<'a, 'b>(
        slf: &'a Bound<'b, Self>,
        f: &'a Bound<'b, PyFunction>,
    ) -> PyResult<Vec<((String, String), Bound<'b, IndexedTableValue>)>> {
        let slf_borrow = slf.borrow();
        let mut result = Vec::new();
        for (key, index) in slf_borrow.keytable.bind(slf.py()).iter() {
            let (key, index): ((String, String), isize) = (key.extract()?, index.extract()?);
            if f.call1((key.clone(),))?.extract()? {
                result.push((
                    key,
                    slf_borrow
                        .indextable
                        .bind(slf.py())
                        .as_any()
                        .get_item(index)?
                        .downcast::<IndexedTableValue>()?
                        .clone(),
                ));
            }
        }
        Ok(result)
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

    fn read_xml(
        slf: &Bound<Self>,
        node: &Bound<PyAny>, // Optional[ET.Element]
        tag: String,
        get_value: Option<Bound<PyFunction>>, // ET.Element -> IndexedTableValue
        get_key: Option<Bound<PyFunction>>,   // IndexedTableValue -> (String, String)
        get_index: Option<Bound<PyFunction>>, // IndexedTableValue -> int
    ) -> PyResult<()> {
        let mut slf_borrow = slf.borrow_mut();
        if node.is_none() {
            return Err(IndexedTableError::new_err(format!(
                "Xml node not present in {}",
                slf_borrow.name
            )));
        }
        for snode in node
            .call_method1(intern!(slf.py(), "findall"), (tag,))?
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
                obj.getattr(intern!(slf.py(), "key"))?
            }
            .extract()?;
            let index: isize = if let Some(get_index) = get_index.as_ref() {
                get_index.call1((obj.clone(),))?
            } else {
                obj.getattr(intern!(slf.py(), "index"))?
            }
            .extract()?;
            slf_borrow.keytable.bind(slf.py()).set_item(key, index)?;
            slf_borrow.indextable.bind(slf.py()).set_item(index, obj)?;
            if index >= slf_borrow.next {
                slf_borrow.next = index + 1
            }
        }
        Ok(())
    }

    fn objectmap<'a, 'b>(
        slf: &'a Bound<'b, Self>,
        p: &'a Bound<'b, PyAny>, // int -> IndexedTableValue, but sometimes non-PyFunction
    ) -> PyResult<BTreeMap<isize, Bound<'b, IndexedTableValue>>> {
        IndexedTable::items(slf)?
            .into_iter()
            .map(|(ix, _)| Ok((ix, p.call1((ix,))?.downcast()?.clone())))
            .collect()
    }

    // Unvalidated
    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        let slf_borrow = slf.borrow();
        let mut lines = Vec::new();
        lines.push(format!("\n{}", slf_borrow.name));
        let reserved = slf_borrow.reserved.bind(slf.py());
        for (ix, obj) in slf_borrow.indextable.bind(slf.py()).iter() {
            let ix: isize = ix.extract()?;
            let obj = obj.downcast::<IndexedTableValue>()?;
            lines.push(format!("{ix:>4} {obj}"));
        }
        if reserved.len() > 0 {
            lines.push(format!("Reserved: {}", reserved.str()?));
        }
        if let Some(cp) = slf_borrow.checkpoint {
            lines.push(format!("Checkpoint: {cp}"));
        }
        Ok(lines.join("\n"))
    }
}
