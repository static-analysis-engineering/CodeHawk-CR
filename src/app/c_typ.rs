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
use once_cell::sync::Lazy;
use pyo3::{exceptions::PyException, intern, prelude::*};

use crate::{
    app::{
        c_attributes::CAttributes, c_dictionary::CDictionary,
        c_dictionary_record::CDictionaryRecord, c_exp::CExp,
    },
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_typ")?;
    module.add_class::<CTyp>()?;
    Ok(module)
}

fn chklogger_info(py: Python, text: String) -> PyResult<()> {
    let chc = PyModule::import_bound(py, intern!(py, "chc"))?;
    let util = chc.getattr(intern!(py, "util"))?;
    let loggingutil = util.getattr(intern!(py, "loggingutil"))?;
    let chklogger = loggingutil.getattr(intern!(py, "chklogger"))?;
    let logger = chklogger.getattr(intern!(py, "logger"))?;
    logger.call_method1(intern!(py, "info"), (text,))?;
    Ok(())
}

/// Base class of all variable types.
#[pyclass(extends = CDictionaryRecord, frozen, subclass)]
pub struct CTyp {
    args: Vec<isize>,
    cd: Py<CDictionary>,
    tags: Vec<String>,
}

const ATTRIBUTE_INDEX: Lazy<BTreeMap<&'static str, usize>> = Lazy::new(|| {
    BTreeMap::from([
        ("tvoid", 0),
        ("tint", 0),
        ("tfloat", 0),
        ("tptr", 1),
        ("tarray", 2),
        ("tfun", 3),
        ("tnamed", 0),
        ("tcomp", 1),
        ("tenum", 0),
        ("tbuiltin-va)-list", 0),
    ])
});

#[pymethods]
impl CTyp {
    #[new]
    fn new(cd: &Bound<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        let ctyp = CTyp {
            args: ixval.args().to_vec(),
            cd: cd.clone().unbind(),
            tags: ixval.tags().to_vec(),
        };
        PyClassInitializer::from(CDictionaryRecord::new(cd.clone().unbind(), ixval))
            .add_subclass(ctyp)
    }

    fn expand<'a, 'b>(slf: &'a Bound<'b, Self>) -> &'a Bound<'b, Self> {
        slf
    }

    fn strip_attributes<'a>(slf: &Bound<'a, Self>) -> PyResult<Bound<'a, Self>> {
        let sg = slf.get();
        let (args, cd, tags) = (&sg.args, sg.cd.bind(slf.py()), &sg.tags);
        let aindex = *ATTRIBUTE_INDEX
            .get(tags[0].as_str())
            .ok_or_else(|| PyException::new_err(format!("no such aindex: {}", tags[0])))?;
        if aindex >= args.len() {
            return Ok(slf.clone());
        } else if args[aindex] == 1 {
            return Ok(slf.clone());
        }
        let newargs = args[..args.len() - 1].to_vec();
        let newtypix = cd
            .call_method1(intern!(slf.py(), "mk_typ_index"), (tags.to_vec(), newargs))?
            .extract::<isize>()?;
        if newtypix != slf.borrow().into_super().into_super().index() {
            let newtyp = cd
                .call_method1(intern!(slf.py(), "get_typ"), (newtypix,))?
                .downcast::<CTyp>()?
                .clone();
            chklogger_info(
                slf.py(),
                format!(
                    "Stripping attributes {} ; changing type from {} to {}",
                    slf.get().attributes_string(slf.py())?,
                    slf.str()?,
                    newtyp.str()?
                ),
            )?;
            Ok(newtyp)
        } else {
            // Would have errored in python implementation because newtyp is not defined
            Err(PyException::new_err(format!(
                "unexpected condition: newtyp not defined"
            )))
        }
    }

    fn get_typ<'a>(&self, py: Python<'a>, ix: isize) -> PyResult<Bound<'a, CTyp>> {
        Ok(self
            .cd
            .bind(py)
            .call_method1(intern!(py, "get_typ"), (ix,))?
            .downcast()?
            .clone())
    }

    // Unvalidated
    fn get_exp<'a>(&self, py: Python<'a>, ix: isize) -> PyResult<Bound<'a, CExp>> {
        Ok(self
            .cd
            .bind(py)
            .call_method1(intern!(py, "get_exp"), (ix,))?
            .downcast()?
            .clone())
    }

    // Unvalidated
    fn get_exp_opt<'a>(&self, py: Python<'a>, ix: isize) -> PyResult<Option<Bound<'a, CExp>>> {
        let v = self
            .cd
            .bind(py)
            .call_method1(intern!(py, "get_exp_opt"), (ix,))?;
        Ok(if v.is_none() {
            None
        } else {
            Some(v.downcast()?.clone())
        })
    }

    #[getter]
    fn attributes<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, CAttributes>> {
        let aindex = *ATTRIBUTE_INDEX
            .get(self.tags[0].as_str())
            .ok_or_else(|| PyException::new_err(format!("no such aindex: {}", self.tags[0])))?;
        let index = if aindex < self.args.len() {
            self.args[aindex]
        } else {
            1
        };
        CDictionary::get_attributes(self.cd.bind(py), index)
    }

    #[getter]
    fn attributes_string<'a>(&self, py: Python<'a>) -> PyResult<String> {
        let attrs = self.attributes(py)?;
        if attrs.get().length() > 0 {
            Ok(format!("[{}]", attrs.str()?))
        } else {
            Ok("".to_string())
        }
    }

    // Unvalidated
    #[getter]
    fn get_opaque_type<'a, 'b>(slf: &'a Bound<'b, Self>) -> &'a Bound<'b, Self> {
        slf
    }

    // Unvalidated
    fn equal<'a>(slf: &Bound<'a, Self>, other: &Bound<'a, Self>) -> PyResult<bool> {
        let expand = intern!(slf.py(), "expand");
        let index = intern!(slf.py(), "index");
        let slf_index: isize = slf.call_method0(expand)?.getattr(index)?.extract()?;
        let other_index: isize = other.call_method0(expand)?.getattr(index)?.extract()?;
        Ok(slf_index == other_index)
    }

    #[getter]
    fn size(&self) -> isize {
        -1000
    }

    #[getter]
    fn is_array(&self) -> bool {
        false
    }

    #[getter]
    fn is_builtin_vaargs(&self) -> bool {
        false
    }

    #[getter]
    fn is_comp(&self) -> bool {
        false
    }

    #[getter]
    fn is_enum(&self) -> bool {
        false
    }

    #[getter]
    fn is_float(&self) -> bool {
        false
    }

    #[getter]
    fn is_function(&self) -> bool {
        false
    }

    #[getter]
    fn is_int(&self) -> bool {
        false
    }

    #[getter]
    fn is_named_type(&self) -> bool {
        false
    }

    #[getter]
    fn is_pointer(&self) -> bool {
        false
    }

    #[getter]
    fn is_struct(&self) -> bool {
        false
    }

    #[getter]
    fn is_void(&self) -> bool {
        false
    }

    #[getter]
    fn is_default_function_prototype(&self) -> bool {
        false
    }

    // Unvalidated
    fn writexml(slf: &Bound<Self>, cnode: &Bound<PyAny>) -> PyResult<()> {
        let set = intern!(cnode.py(), "set");
        cnode.call_method1(
            set,
            (
                intern!(slf.py(), "ix"),
                format!("{}", slf.borrow().into_super().into_super().index()),
            ),
        )?;
        cnode.call_method1(set, (intern!(slf.py(), "tags"), slf.get().tags.join(",")))?;
        cnode.call_method1(
            set,
            (intern!(slf.py(), "args"), slf.get().args.iter().join(",")),
        )?;
        Ok(())
    }

    // Unvalidated
    #[pyo3(name = "__str__")]
    fn str(&self) -> String {
        format!("typebase: {}", self.tags[0])
    }

    // Unvalidated
    fn to_dict(&self) -> BTreeMap<&'static str, &'static str> {
        BTreeMap::from([("base", "type")])
    }

    // Unvalidated
    fn to_idict(&self, py: Python) -> BTreeMap<&'static str, Py<PyAny>> {
        BTreeMap::from([
            ("t", self.tags.to_object(py)),
            ("a", self.args.to_object(py)),
        ])
    }
}
