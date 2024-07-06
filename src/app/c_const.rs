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
use pyo3::{intern, prelude::*};

use crate::{
    app::{
        c_dictionary::CDictionary,
        c_dictionary_record::{CDictionaryRecord, CDictionaryRegistryEntry},
    },
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_const")?;
    module.add_class::<CConst>()?;
    module.add_class::<CConstInt>()?;
    module.add_class::<CConstStr>()?;
    module.add_class::<CConstWStr>()?;
    module.add_class::<CConstChr>()?;
    module.add_class::<CConstReal>()?;
    module.add_class::<CConstEnum>()?;
    module.add_class::<CStringConstant>()?;
    Ok(module)
}

/// Constant expression.
#[pyclass(extends = CDictionaryRecord, frozen, subclass)]
pub struct CConst {}

#[pymethods]
impl CConst {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDictionaryRecord::new(cd, ixval)).add_subclass(CConst {})
    }

    fn get_exp(slf: PyRef<Self>, py: Python, ix: isize) -> PyResult<Py<PyAny>> {
        slf.into_super()
            .cd()
            .call_method1(py, intern!(py, "get_exp"), (ix,))
    }

    fn get_strings(&self) -> Vec<String> {
        vec![]
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
    fn is_wstr(&self) -> bool {
        false
    }

    #[getter]
    fn is_chr(&self) -> bool {
        false
    }

    #[getter]
    fn is_real(&self) -> bool {
        false
    }

    #[getter]
    fn is_enum(&self) -> bool {
        false
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>) -> String {
        slf.into_super().into_super().tags()[0].clone()
    }
}

/// Constant integer.
///
/// - tags[1]: string representation of value
/// - tags[2]: ikind
#[pyclass(extends = CConst, frozen, subclass)]
struct CConstInt {}

#[pymethods]
impl CConstInt {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CConst::new(cd, ixval)).add_subclass(CConstInt {})
    }

    #[getter]
    fn intvalue(slf: PyRef<Self>) -> PyResult<isize> {
        Ok(slf.into_super().into_super().into_super().tags()[1].parse()?)
    }

    #[getter]
    fn ikind(slf: PyRef<Self>) -> String {
        slf.into_super().into_super().into_super().tags()[2].clone()
    }

    #[getter]
    fn is_int(&self) -> bool {
        true
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>) -> PyResult<String> {
        // TODO reference the value directly
        Ok(format!("{}", CConstInt::intvalue(slf)?))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CConst, CConstInt>("int") }

/// Constant string
///
/// - args[0]: string index
#[pyclass(extends = CConst, frozen, subclass)]
struct CConstStr {}

#[pymethods]
impl CConstStr {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CConst::new(cd, ixval)).add_subclass(CConstStr {})
    }

    #[getter]
    fn stringvalue(slf: &Bound<Self>) -> PyResult<String> {
        let dict_record = slf.borrow().into_super().into_super();
        let decls = dict_record.cd().bind(slf.py()).clone();
        let arg0 = dict_record.into_super().args()[0];
        decls
            .call_method1(intern!(slf.py(), "get_string"), (arg0,))?
            .extract()
    }

    fn get_strings(slf: &Bound<Self>) -> PyResult<Vec<String>> {
        Ok(vec![CConstStr::stringvalue(slf)?])
    }

    #[getter]
    fn is_str(&self) -> bool {
        true
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> PyResult<String> {
        let strg = CConstStr::stringvalue(slf)?;
        if strg.len() > 25 {
            Ok(format!("{}-char string", strg.len()))
        } else {
            Ok(format!("str({})", strg))
        }
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CConst, CConstStr>("str") }

/// Constant wide string (represented as a sequence of int64 integers)
///
/// - tags[1..]: string representation of int64 integers
#[pyclass(extends = CConst, frozen, subclass)]
struct CConstWStr {}

#[pymethods]
impl CConstWStr {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CConst::new(cd, ixval)).add_subclass(CConstWStr {})
    }

    #[getter]
    fn stringvalue(slf: PyRef<Self>) -> String {
        slf.into_super().into_super().into_super().tags()[1..]
            .iter()
            .join("-")
    }

    #[getter]
    fn is_wstr(&self) -> bool {
        true
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>) -> String {
        format!("wstr({})", CConstWStr::stringvalue(slf))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CConst, CConstWStr>("wstr") }

/// Constant character.
///
/// - args[0]: char code
#[pyclass(extends = CConst, frozen, subclass)]
struct CConstChr {}

#[pymethods]
impl CConstChr {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CConst::new(cd, ixval)).add_subclass(CConstChr {})
    }

    #[getter]
    fn chrvalue(slf: PyRef<Self>) -> isize {
        slf.into_super().into_super().into_super().args()[0]
    }

    #[getter]
    fn is_chr(&self) -> bool {
        true
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>) -> String {
        format!("chr({})", CConstChr::chrvalue(slf))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CConst, CConstChr>("chr") }

/// Constant real number.
///
/// - tags[1]: string representation of real
/// - tags[2]: fkind
#[pyclass(extends = CConst, frozen, subclass)]
struct CConstReal {}

#[pymethods]
impl CConstReal {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CConst::new(cd, ixval)).add_subclass(CConstReal {})
    }

    #[getter]
    fn realvalue(slf: PyRef<Self>) -> PyResult<f64> {
        Ok(slf.into_super().into_super().into_super().tags()[1].parse()?)
    }

    #[getter]
    fn fkind(slf: PyRef<Self>) -> String {
        slf.into_super().into_super().into_super().tags()[2].clone()
    }

    #[getter]
    fn is_real(&self) -> bool {
        true
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>) -> PyResult<String> {
        Ok(format!("{}", CConstReal::realvalue(slf)?))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CConst, CConstReal>("real") }

/// Constant enumeration value.
///
/// - tags[1]: enum name
/// - tags[2]: enum item name
///
/// - args[0]: exp
#[pyclass(extends = CConst, frozen, subclass)]
struct CConstEnum {}

#[pymethods]
impl CConstEnum {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CConst::new(cd, ixval)).add_subclass(CConstEnum {})
    }

    #[getter]
    fn enum_name(slf: PyRef<Self>) -> String {
        slf.into_super().into_super().into_super().tags()[1].clone()
    }

    #[getter]
    fn item_name(slf: PyRef<Self>) -> String {
        slf.into_super().into_super().into_super().tags()[2].clone()
    }

    #[getter]
    fn exp(slf: Py<Self>, py: Python) -> PyResult<Py<PyAny>> {
        let c_const_ref = slf.borrow(py).into_super();
        let arg0 = slf.borrow(py).into_super().into_super().into_super().args()[0];
        CConst::get_exp(c_const_ref, py, arg0)
    }

    #[getter]
    fn is_enum(&self) -> bool {
        true
    }

    #[pyo3(name = "__str__")]
    fn str(slf: Py<Self>, py: Python) -> PyResult<String> {
        let enum_name = CConstEnum::enum_name(slf.borrow(py));
        let item_name = CConstEnum::item_name(slf.borrow(py));
        let exp = CConstEnum::exp(slf, py)?;
        Ok(format!("{}: {}({})", enum_name, item_name, exp))
    }
}

inventory::submit! { CDictionaryRegistryEntry::python_type::<CConst, CConstEnum>("enum") }

// Seems unused
/// Constant string value
///
/// - tags[0]: string value or hexadecimal representation of string value
/// - tags[1]: 'x' (optional) if string value is represented in hexadecimal
///
/// - args[0] length of original string
#[pyclass(extends = CDictionaryRecord, frozen, subclass)]
struct CStringConstant {}

#[pymethods]
impl CStringConstant {
    #[new]
    fn new(cd: Py<CDictionary>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDictionaryRecord::new(cd, ixval)).add_subclass(CStringConstant {})
    }

    #[getter]
    fn stringvalue(slf: PyRef<Self>) -> String {
        let it_val = slf.into_super().into_super();
        let tags = it_val.tags();
        if tags.is_empty() {
            "".to_string()
        } else {
            tags[0].clone()
        }
    }

    #[getter]
    fn string_length(slf: PyRef<Self>) -> isize {
        slf.into_super().into_super().args()[0]
    }

    #[getter]
    fn is_hex(slf: PyRef<Self>) -> bool {
        slf.into_super().into_super().tags().len() > 1
    }

    #[pyo3(name = "__str__")]
    fn str(slf: &Bound<Self>) -> String {
        if CStringConstant::is_hex(slf.borrow()) {
            format!(
                "({}-char string",
                CStringConstant::string_length(slf.borrow())
            )
        } else {
            CStringConstant::stringvalue(slf.borrow())
        }
    }
}
