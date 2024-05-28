use pyo3::prelude::*;

use crate::app::c_dictionary_record::CDictionaryRecord;
use crate::util::indexed_table::IndexedTableValue;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_const")?;
    module.add_class::<CConst>()?;
    module.add_class::<CConstInt>()?;
    Ok(module)
}

/// Constant expression.
#[derive(Clone)]
#[pyclass(extends = CDictionaryRecord, frozen, subclass)]
struct CConst {}

#[pymethods]
impl CConst {
    #[new]
    fn new(cd: Py<PyAny>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CDictionaryRecord::new(cd, ixval)).add_subclass(CConst {})
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
#[derive(Clone)]
#[pyclass(extends = CConst, frozen, subclass)]
struct CConstInt {}

#[pymethods]
impl CConstInt {
    #[new]
    fn new(cd: Py<PyAny>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
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
