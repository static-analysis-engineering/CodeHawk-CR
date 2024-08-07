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
        c_attributes::CAttributes,
        c_comp_info::CCompInfo,
        c_const::CConstInt,
        c_dictionary::CDictionary,
        c_dictionary_record::{CDictionaryRecord, CDictionaryRegistryEntry},
        c_exp::{CExp, CExpConst},
    },
    util::indexed_table::IndexedTableValue,
};

pyo3::import_exception!(chcc.util.fileutil, CHCError);

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_typ")?;
    module.add_class::<CTyp>()?;
    module.add_class::<CTypArray>()?;
    module.add_class::<CTypBuiltinVaargs>()?;
    module.add_class::<CTypComp>()?;
    module.add_class::<CTypEnum>()?;
    module.add_class::<CTypFloat>()?;
    module.add_class::<CTypInt>()?;
    module.add_class::<CTypNamed>()?;
    module.add_class::<CTypPtr>()?;
    module.add_class::<CTypVoid>()?;
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

const FLOAT_NAMES: Lazy<BTreeMap<&'static str, &'static str>> = Lazy::new(|| {
    BTreeMap::from([
        ("float", "float"),
        ("fdouble", "double"),
        ("flongdouble", "long double"),
    ])
});

const INTEGER_NAMES: Lazy<BTreeMap<&'static str, &'static str>> = Lazy::new(|| {
    BTreeMap::from([
        ("ichar", "char"),
        ("ischar", "signed char"),
        ("iuchar", "unsigned char"),
        ("ibool", "bool"),
        ("iint", "int"),
        ("iuint", "unsigned int"),
        ("ishort", "short"),
        ("iushort", "unsigned short"),
        ("ilong", "long"),
        ("iulong", "unsigned long"),
        ("ilonglong", "long long"),
        ("iulonglong", "unsigned long long"),
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

/// Void type.
///
/// * args[0]: attributes
#[pyclass(extends = CTyp, frozen, subclass)]
pub struct CTypVoid {}

#[pymethods]
impl CTypVoid {
    #[new]
    fn new(cd: &Bound<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CTyp::new(cd, ixval)).add_subclass(CTypVoid {})
    }

    #[getter]
    fn is_void(&self) -> bool {
        true
    }

    // Unvalidated
    fn to_dict(&self) -> BTreeMap<&'static str, &'static str> {
        BTreeMap::from([("base", "void")])
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        let attributes = slf.borrow().into_super().attributes(slf.py())?.str()?;
        Ok(format!("void[{}]", attributes))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CTyp, CTypVoid>("tvoid") }

/// Integer type.
///
/// * tags[1]: ikind
///
/// * args[0]: index of attributes in cdictionary
#[pyclass(extends = CTyp, frozen, subclass)]
pub struct CTypInt {
    ikind: String,
}

#[pymethods]
impl CTypInt {
    #[new]
    fn new(cd: &Bound<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        let typint = CTypInt {
            ikind: ixval.tags()[1].clone(),
        };
        PyClassInitializer::from(CTyp::new(cd, ixval)).add_subclass(typint)
    }

    #[getter]
    fn is_int(&self) -> bool {
        true
    }

    // Unvalidated
    #[getter]
    fn size(&self) -> PyResult<isize> {
        let binding = INTEGER_NAMES;
        let name = binding
            .get(self.ikind.as_str())
            .ok_or_else(|| PyException::new_err(format!("unknown type '{}'", self.ikind)))?;
        Ok(if name.contains("char") {
            1
        } else {
            4 // TBD: adjust for other kinds
        })
    }

    // Unvalidated
    #[getter]
    fn ikind(&self) -> &str {
        self.ikind.as_str()
    }

    // Unvalidated
    fn to_dict(&self) -> BTreeMap<&'static str, &str> {
        BTreeMap::from([("base", "int"), ("kind", self.ikind.as_str())])
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        let slf_borrow = slf.borrow();
        let binding = INTEGER_NAMES;
        let name = binding
            .get(slf_borrow.ikind.as_str())
            .ok_or_else(|| PyException::new_err(format!("unknown type '{}'", slf_borrow.ikind)))?;
        let attributes_string = slf.borrow().into_super().attributes_string(slf.py())?;
        Ok(format!("{name}{attributes_string}"))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CTyp, CTypInt>("tint") }

/// Float type.
///
/// * tags[1]: fkind
///
/// * args[0]: attributes
#[pyclass(extends = CTyp, frozen, subclass)]
pub struct CTypFloat {
    fkind: String,
}

#[pymethods]
impl CTypFloat {
    #[new]
    fn new(cd: &Bound<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        let typfloat = CTypFloat {
            fkind: ixval.tags()[1].clone(),
        };
        PyClassInitializer::from(CTyp::new(cd, ixval)).add_subclass(typfloat)
    }

    #[getter]
    fn is_float(&self) -> bool {
        true
    }

    // Unvalidated
    #[getter]
    fn size(&self) -> isize {
        4 // TBD: adjust for kind
    }

    // Unvalidated
    #[getter]
    fn fkind(&self) -> &str {
        self.fkind.as_str()
    }

    // Unvalidated
    fn to_dict(&self) -> BTreeMap<&'static str, &str> {
        BTreeMap::from([("base", "float"), ("kind", self.fkind.as_str())])
    }

    #[pyo3(name = "__str__")]
    fn str(&self) -> PyResult<&'static str> {
        FLOAT_NAMES
            .get(self.fkind.as_str())
            .cloned()
            .ok_or_else(|| PyException::new_err(format!("unknown type '{}'", self.fkind)))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CTyp, CTypFloat>("tfloat") }

/// Type definition
///
/// * tags[1]: tname
///
/// * args[0]: attributes
#[pyclass(extends = CTyp, frozen, subclass)]
pub struct CTypNamed {
    cd: Py<CDictionary>,
    #[pyo3(get)]
    name: String,
}

#[pymethods]
impl CTypNamed {
    #[new]
    fn new(cd: &Bound<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        let typnamed = CTypNamed {
            cd: cd.clone().unbind(),
            name: ixval.tags()[1].clone(),
        };
        PyClassInitializer::from(CTyp::new(cd, ixval)).add_subclass(typnamed)
    }

    fn expand<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, CTyp>> {
        Ok(self
            .cd
            .bind(py)
            .getattr(intern!(py, "decls"))?
            .call_method1(intern!(py, "expand"), (self.name.as_str(),))?
            .downcast()?
            .clone())
    }

    // Unvalidated
    #[getter]
    fn size(&self, py: Python) -> PyResult<isize> {
        Ok(self.expand(py)?.getattr(intern!(py, "size"))?.extract()?)
    }

    #[getter]
    fn is_named_type(&self) -> bool {
        true
    }

    // Unvalidated
    #[getter]
    fn get_opaque_type<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, CTyp>> {
        Ok(self
            .expand(py)?
            .call_method0(intern!(py, "get_opaque_type"))?
            .downcast()?
            .clone())
    }

    // Unvalidated
    fn to_dict<'a>(&self, py: Python<'a>) -> PyResult<BTreeMap<&'static str, Py<PyAny>>> {
        Ok(BTreeMap::from([
            ("base", "named".into_py(py)),
            ("name", self.name.as_str().into_py(py)),
            (
                "expand",
                self.expand(py)?
                    .call_method0(intern!(py, "to_dict"))?
                    .unbind(),
            ),
        ]))
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        Ok(format!(
            "{}{}",
            slf.borrow().name,
            slf.borrow().as_super().attributes_string(slf.py())?
        ))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CTyp, CTypNamed>("tnamed") }

/// Struct type (composite type; also includes union)
///
/// * tags[0]: struct name
///
/// * args[0]: ckey
/// * args[1]: index of attributes in cdictionary
#[pyclass(extends = CTyp, frozen, subclass)]
pub struct CTypComp {
    cd: Py<CDictionary>,
    #[pyo3(get)]
    ckey: isize,
}

#[pymethods]
impl CTypComp {
    #[new]
    fn new(cd: &Bound<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        let typcomp = CTypComp {
            cd: cd.clone().unbind(),
            ckey: ixval.args()[0],
        };
        PyClassInitializer::from(CTyp::new(cd, ixval)).add_subclass(typcomp)
    }

    #[getter]
    fn compinfo<'a>(slf: &Bound<'a, Self>) -> PyResult<Bound<'a, CCompInfo>> {
        Ok(slf
            .getattr(intern!(slf.py(), "decls"))?
            .call_method1(
                intern!(slf.py(), "get_compinfo_by_ckey"),
                (slf.borrow().ckey,),
            )?
            .downcast()?
            .clone())
    }

    #[getter]
    fn name(slf: &Bound<Self>) -> PyResult<String> {
        CCompInfo::name(&Self::compinfo(slf)?)
    }

    #[getter]
    fn is_struct(slf: &Bound<Self>) -> PyResult<bool> {
        Ok(CCompInfo::is_struct(Self::compinfo(slf)?.borrow()))
    }

    // Unvalidated
    #[getter]
    fn size(slf: &Bound<Self>) -> PyResult<isize> {
        CCompInfo::size(&Self::compinfo(slf)?)
    }

    #[getter]
    fn is_comp(&self) -> bool {
        true
    }

    // Unvalidated
    fn get_opaque_type<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, CTyp>> {
        let tags = ["tvoid"];
        let args: [isize; 0] = [];
        let cd = self.cd.bind(py);
        let typ_index = cd.call_method1(intern!(py, "mk_typ_index"), (tags, args))?;
        Ok(self
            .cd
            .bind(py)
            .call_method1(intern!(py, "get_typ"), (typ_index,))?
            .downcast()?
            .clone())
    }

    // Unvalidated
    fn to_dict(slf: &Bound<Self>) -> PyResult<BTreeMap<&'static str, Py<PyAny>>> {
        let py = slf.py();
        Ok(BTreeMap::from([
            ("base", "struct".into_py(py)),
            (
                "kind",
                if Self::is_struct(slf)? {
                    "struct"
                } else {
                    "union "
                }
                .into_py(py),
            ),
            ("name", Self::name(slf)?.into_py(py)),
            ("key", slf.borrow().ckey.into_py(py)),
        ]))
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        let typ = if Self::is_struct(slf)? {
            "struct"
        } else {
            "union"
        };
        Ok(format!("{typ} {}({})", Self::name(slf)?, slf.borrow().ckey))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CTyp, CTypComp>("tcomp") }

/// Enum type.
///
/// * tags[1]: name of enum (ename)
/// * args[0]: index of attributes in cdictionary
#[pyclass(extends = CTyp, frozen, subclass)]
pub struct CTypEnum {
    cd: Py<CDictionary>,
    #[pyo3(get)]
    name: String,
}

#[pymethods]
impl CTypEnum {
    #[new]
    fn new(cd: &Bound<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        let typenum = CTypEnum {
            cd: cd.clone().unbind(),
            name: ixval.tags()[1].clone(),
        };
        PyClassInitializer::from(CTyp::new(cd, ixval)).add_subclass(typenum)
    }

    // Unvalidated
    #[getter]
    fn size(&self) -> isize {
        4
    }

    #[getter]
    fn is_enum(&self) -> bool {
        true
    }

    // Unvalidated
    fn get_opaque_type<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, CTyp>> {
        let tags = ["tint", "iint"];
        let args: [isize; 0] = [];
        let cd = self.cd.bind(py);
        let typ_index = cd.call_method1(intern!(py, "mk_typ_index"), (tags, args))?;
        Ok(self
            .cd
            .bind(py)
            .call_method1(intern!(py, "get_typ"), (typ_index,))?
            .downcast()?
            .clone())
    }

    // Unvalidated
    fn to_dict(&self) -> BTreeMap<&'static str, &str> {
        BTreeMap::from([("base", "struct"), ("name", self.name.as_str())])
    }

    #[pyo3(name = "__str__")]
    fn str(&self) -> String {
        format!("enum {}", self.name)
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CTyp, CTypEnum>("tenum") }

// Unvalidated
#[pyclass(extends = CTyp, frozen, subclass)]
pub struct CTypBuiltinVaargs {}

#[pymethods]
impl CTypBuiltinVaargs {
    #[new]
    fn new(cd: &Bound<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        let vaargs = CTypBuiltinVaargs {};
        PyClassInitializer::from(CTyp::new(cd, ixval)).add_subclass(vaargs)
    }

    #[getter]
    fn is_builtin_vaargs(&self) -> bool {
        true
    }

    fn to_dict(&self) -> BTreeMap<&'static str, &'static str> {
        BTreeMap::from([("base", "builtin_vaargs")])
    }

    #[pyo3(name = "__str__")]
    fn str(&self) -> &'static str {
        "tbuiltin_va_args"
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CTyp, CTypBuiltinVaargs>("tbuiltinvaargs") }
inventory::submit! { CDictionaryRegistryEntry::python_type::<CTyp, CTypBuiltinVaargs>("tbuiltin-va-list") }

/// Pointer type
///
/// * args[0]: index of pointed-to type in cdictionary
/// * args[1]: index of attributes in cdictionary
#[pyclass(extends = CTyp, frozen, subclass)]
pub struct CTypPtr {
    cd: Py<CDictionary>,
    pointed_to_index: isize,
}

#[pymethods]
impl CTypPtr {
    #[new]
    fn new(cd: &Bound<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        let ptr = CTypPtr {
            cd: cd.clone().unbind(),
            pointed_to_index: ixval.args()[0],
        };
        PyClassInitializer::from(CTyp::new(cd, ixval)).add_subclass(ptr)
    }

    #[getter]
    fn pointedto_type<'a>(slf: PyRef<Self>, py: Python<'a>) -> PyResult<Bound<'a, CTyp>> {
        slf.as_super().get_typ(py, slf.pointed_to_index)
    }

    // Unvalidated
    #[getter]
    fn size(&self) -> isize {
        4
    }

    #[getter]
    fn is_pointer(&self) -> bool {
        true
    }

    // Unvalidated
    fn get_opaque_type<'a>(slf: &Bound<'a, Self>) -> PyResult<Bound<'a, CTyp>> {
        let slf_borrow = slf.borrow();
        let py = slf.py();
        let tgttype = CTypPtr::pointedto_type(slf.borrow(), py)?;
        let tags = ["tptr"];
        let cd = slf_borrow.cd.bind(py);
        let index_typ = cd.call_method1(intern!(py, "index_typ"), (tgttype,))?;
        let args = [index_typ.extract::<isize>()?];
        let typ_index = cd.call_method1(intern!(py, "mk_typ_index"), (tags, args))?;
        Ok(cd
            .call_method1(intern!(py, "get_typ"), (typ_index,))?
            .downcast()?
            .clone())
    }

    // Unvalidated
    fn to_dict(slf: &Bound<Self>) -> PyResult<BTreeMap<&'static str, Py<PyAny>>> {
        let py = slf.py();
        Ok(BTreeMap::from([
            ("base", "ptr".into_py(py)),
            (
                "tgt",
                Self::pointedto_type(slf.borrow(), py)?
                    .call_method0(intern!(py, "to_dict"))?
                    .unbind(),
            ),
        ]))
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>, py: Python) -> PyResult<String> {
        Ok(format!("({})", Self::pointedto_type(slf, py)?.str()?))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CTyp, CTypPtr>("tptr") }

/// Array type
///
/// * args[0]: index of base type in cdictionary
/// * args[1]: index of size expression in cdictionary (optional)
/// * args[2]: index of attributes in cdictionary
#[pyclass(extends = CTyp, frozen, subclass)]
pub struct CTypArray {
    cd: Py<CDictionary>,
    base_type_index: isize,
    size_expression_index: isize,
}

#[pymethods]
impl CTypArray {
    #[new]
    fn new(cd: &Bound<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        let ptr = CTypArray {
            cd: cd.clone().unbind(),
            base_type_index: ixval.args()[0],
            size_expression_index: ixval.args()[1],
        };
        PyClassInitializer::from(CTyp::new(cd, ixval)).add_subclass(ptr)
    }

    #[getter]
    fn array_basetype<'a>(slf: PyRef<Self>, py: Python<'a>) -> PyResult<Bound<'a, CTyp>> {
        slf.as_super().get_typ(py, slf.base_type_index)
    }

    #[getter]
    fn array_size_expr<'a>(slf: PyRef<Self>, py: Python<'a>) -> PyResult<Bound<'a, CExp>> {
        if slf.size_expression_index >= 0 {
            slf.as_super().get_exp(py, slf.size_expression_index)
        } else {
            Err(CHCError::new_err("Array does not have a size"))
        }
    }

    fn has_array_size_expr(&self) -> bool {
        self.size_expression_index >= 0
    }

    // Unvalidated
    #[getter]
    fn size(slf: &Bound<Self>) -> isize {
        if !slf.borrow().has_array_size_expr() {
            return -1000;
        }
        (|| {
            let array_size_const = Self::array_size_expr(slf.borrow(), slf.py())?
                .downcast::<CExpConst>()?
                .clone();
            let array_size_int = CExpConst::constant(&array_size_const)?
                .downcast::<CConstInt>()?
                .clone();
            CConstInt::intvalue(array_size_int.borrow())
        })()
        .unwrap_or(-1000)
    }

    #[getter]
    fn is_array(&self) -> bool {
        true
    }

    // Unvalidated
    fn get_opaque_type<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, CTyp>> {
        let tags = ["tvoid"];
        let args: [isize; 0] = [];
        let cd = self.cd.bind(py);
        let typ_index = cd.call_method1(intern!(py, "mk_typ_index"), (tags, args))?;
        Ok(self
            .cd
            .bind(py)
            .call_method1(intern!(py, "get_typ"), (typ_index,))?
            .downcast()?
            .clone())
    }

    // Unvalidated
    fn to_dict(slf: &Bound<Self>) -> PyResult<BTreeMap<&'static str, Py<PyAny>>> {
        let py = slf.py();
        let mut map = BTreeMap::from([
            ("base", "array".into_py(py)),
            (
                "elem",
                Self::array_basetype(slf.borrow(), py)?
                    .call_method0(intern!(py, "to_dict"))?
                    .unbind(),
            ),
        ]);
        if slf.borrow().has_array_size_expr()
            && Self::array_basetype(slf.borrow(), py)?
                .getattr(intern!(py, "is_constant"))?
                .extract()?
        {
            map.insert(
                "size",
                Self::array_size_expr(slf.borrow(), py)?
                    .str()?
                    .into_any()
                    .unbind(),
            );
        }
        Ok(map)
    }

    // Unvalidated
    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        let py = slf.py();
        let size = Self::array_size_expr(slf.borrow(), py)?;
        let ssize = if size.is_none() {
            "?".to_string()
        } else {
            size.str()?.extract()?
        };
        Ok(format!(
            "{}[{ssize}]",
            Self::array_basetype(slf.borrow(), py)?.str()?
        ))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CTyp, CTypArray>("tarray") }
