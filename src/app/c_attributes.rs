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
use std::borrow::Cow;

use pyo3::{intern, prelude::*};

use crate::{
    app::{
        c_dictionary::CDictionary,
        c_dictionary_record::{
            CDictionaryRecord, CDictionaryRecordTrait, CDictionaryRegistryEntry,
        },
    },
    util::indexed_table::{
        inherit_indexed_table_value_trait, IndexedTableValue, IndexedTableValueTrait,
    },
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_attributes")?;
    module.add_class::<CAttr>()?;
    module.add_class::<CAttrCons>()?;
    module.add_class::<CAttrInt>()?;
    module.add_class::<CAttrStr>()?;
    module.add_class::<CAttribute>()?;
    module.add_class::<CAttributes>()?;
    Ok(module)
}

/// Attribute that comes with a C type.
#[pyclass(extends = CDictionaryRecord, frozen, subclass)]
pub struct CAttr {}

inherit_indexed_table_value_trait!(CAttr);

#[pymethods]
impl CAttr {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDictionaryRecord::new(cd, ixval)).add_subclass(CAttr {})
    }

    #[getter]
    fn is_int(&self) -> bool {
        false
    }

    #[getter]
    fn is_str(&self) -> bool {
        false
    }

    #[getter]
    fn is_cons(&self) -> bool {
        false
    }

    #[getter]
    fn is_sizeof(&self) -> bool {
        false
    }

    #[getter]
    fn is_sizeofe(&self) -> bool {
        false
    }

    #[getter]
    fn is_sizeofs(&self) -> bool {
        false
    }

    #[getter]
    fn is_alignof(&self) -> bool {
        false
    }

    #[getter]
    fn is_alignofe(&self) -> bool {
        false
    }

    #[getter]
    fn is_alignofs(&self) -> bool {
        false
    }

    #[getter]
    fn is_unop(&self) -> bool {
        false
    }

    #[getter]
    fn is_binop(&self) -> bool {
        false
    }

    #[getter]
    fn is_dot(&self) -> bool {
        false
    }

    #[getter]
    fn is_star(&self) -> bool {
        false
    }

    #[getter]
    fn is_addrof(&self) -> bool {
        false
    }

    #[getter]
    fn is_index(&self) -> bool {
        false
    }

    #[getter]
    fn is_question(&self) -> bool {
        false
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>) -> String {
        format!("attrparam:{}", slf.tags()[0])
    }
}

/// Integer attribute.
///
/// args[0]: integer value
#[pyclass(extends = CAttr, frozen, subclass)]
struct CAttrInt {}

inherit_indexed_table_value_trait!(CAttrInt);

#[pymethods]
impl CAttrInt {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CAttr::new(cd, ixval)).add_subclass(CAttrInt {})
    }

    #[getter]
    fn intvalue(slf: PyRef<Self>) -> isize {
        slf.args()[0]
    }

    #[getter]
    fn is_int(&self) -> bool {
        true
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>) -> String {
        format!("aint({})", CAttrInt::intvalue(slf))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CAttr, CAttrInt>("aint") }

/// String attribute.
///
/// * args[0]: index in string table of string attribute
#[pyclass(extends = CAttr, frozen, subclass)]
struct CAttrStr {}

inherit_indexed_table_value_trait!(CAttrStr);

// Unvalidated
#[pymethods]
impl CAttrStr {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CAttr::new(cd, ixval)).add_subclass(CAttrStr {})
    }

    #[getter]
    fn stringvalue(slf: &Bound<Self>) -> PyResult<String> {
        let py = slf.py();
        slf.borrow()
            .into_super()
            .into_super()
            .cd()
            .call_method1(py, intern!(py, "get_string"), (slf.args()[0],))?
            .extract(py)
    }

    #[getter]
    fn is_str(&self) -> bool {
        true
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        Ok(format!("astr({})", CAttrStr::stringvalue(slf)?))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CAttr, CAttrStr>("astr") }

/// Constructed attributes.
///
/// * tags[1]: name
/// * args[0..]: indices of attribute parameters in cdictionary.
#[pyclass(extends = CAttr, frozen, subclass)]
struct CAttrCons {}

inherit_indexed_table_value_trait!(CAttrCons);

#[pymethods]
impl CAttrCons {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CAttr::new(cd, ixval)).add_subclass(CAttrCons {})
    }

    #[getter]
    fn name(slf: PyRef<Self>) -> String {
        slf.into_super().tags()[1].to_string()
    }

    #[getter]
    fn params<'a>(slf: &Bound<'a, Self>) -> PyResult<Vec<Bound<'a, CAttr>>> {
        let cd = slf
            .borrow()
            .into_super()
            .into_super()
            .cd()
            .bind(slf.py())
            .clone();
        slf.args()
            .iter()
            .map(|i| CDictionary::get_attrparam(&cd, *i))
            .collect()
    }

    #[getter]
    fn is_cons(&self) -> bool {
        true
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>) -> String {
        format!("acons({})", CAttrCons::name(slf))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CAttr, CAttrCons>("acons") }

#[pyclass(extends = CDictionaryRecord, frozen)]
pub struct CAttribute {
    name: String,
    cd: Py<CDictionary>,
    args: Vec<isize>,
}

#[pymethods]
impl CAttribute {
    #[new]
    pub fn new(cd: &Bound<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        let c_attribute = CAttribute {
            name: ixval.tags()[0].clone(),
            cd: cd.clone().unbind(),
            args: ixval.args().to_vec(),
        };
        PyClassInitializer::from(CDictionaryRecord::new(cd.clone().unbind(), ixval))
            .add_subclass(c_attribute)
    }

    #[getter]
    fn name(&self) -> &str {
        self.name.as_str()
    }

    #[getter]
    fn params<'a>(&self, py: Python<'a>) -> PyResult<Vec<Bound<'a, CAttr>>> {
        self.args
            .iter()
            .map(|i| CDictionary::get_attrparam(self.cd.bind(py), *i))
            .collect()
    }

    #[pyo3(name = "__str__")]
    fn str(&self, py: Python) -> PyResult<String> {
        let params = self
            .params(py)?
            .into_iter()
            .map(|b| Ok(b.str()?.extract()?))
            .collect::<PyResult<Vec<String>>>()?;
        Ok(format!("{}: {}", self.name, params.join(",")))
    }
}

impl CDictionaryRecordTrait for CAttribute {
    fn new(cd: &Bound<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        Self::new(cd, ixval)
    }
}

#[pyclass(extends = CDictionaryRecord, frozen)]
pub struct CAttributes {
    args: Vec<isize>,
    cd: Py<CDictionary>,
}

#[pymethods]
impl CAttributes {
    #[new]
    pub fn new(cd: &Bound<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        let attributes = CAttributes {
            args: ixval.args().to_vec(),
            cd: cd.clone().unbind(),
        };
        PyClassInitializer::from(CDictionaryRecord::new(cd.clone().unbind(), ixval))
            .add_subclass(attributes)
    }

    #[getter]
    fn attributes<'a>(&self, py: Python<'a>) -> PyResult<Vec<Bound<'a, CAttribute>>> {
        self.args
            .iter()
            .map(|i| CDictionary::get_attribute(self.cd.bind(py), *i))
            .collect()
    }

    #[getter]
    pub fn length(&self) -> usize {
        self.args.len()
    }

    #[pyo3(name = "__str__")]
    fn str(&self, py: Python) -> PyResult<String> {
        let attributes = self
            .attributes(py)?
            .into_iter()
            .map(|b| Ok(b.str()?.extract()?))
            .collect::<PyResult<Vec<String>>>()?;
        Ok(attributes.join(","))
    }
}

impl CDictionaryRecordTrait for CAttributes {
    fn new(cd: &Bound<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        Self::new(cd, ixval)
    }
}
