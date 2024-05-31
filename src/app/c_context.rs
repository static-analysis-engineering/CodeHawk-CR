use itertools::Itertools;
use pyo3::{exceptions::PyException, intern, prelude::*};

use crate::util::indexed_table::IndexedTableValue;

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_context")?;
    module.add_class::<CContextDictionaryRecord>()?;
    module.add_class::<CContextNode>()?;
    module.add_class::<CfgContext>()?;
    module.add_class::<ExpContext>()?;
    module.add_class::<ProgramContext>()?;
    Ok(module)
}

pyo3::import_exception!(chcc.util.fileutil, CHCError);

#[derive(Clone)]
#[pyclass(extends = IndexedTableValue, frozen, subclass)]
pub struct CContextDictionaryRecord {
    #[pyo3(get)]
    cxd: Py<PyAny>,
}

#[pymethods]
impl CContextDictionaryRecord {
    #[new]
    pub fn new(
        cxd: Py<PyAny>,
        ixval: IndexedTableValue,
    ) -> (CContextDictionaryRecord, IndexedTableValue) {
        (CContextDictionaryRecord { cxd }, ixval)
    }

    #[pyo3(name = "__str__")]
    pub fn str(slf: Py<Self>, py: Python) -> PyResult<String> {
        Ok(format!(
            "context-record: {}",
            slf.getattr(py, intern!(py, "key"))?
        ))
    }
}

impl CContextDictionaryRecord {
    fn cxd(&self) -> Py<PyAny> {
        self.cxd.clone()
    }
}

/// Node in an expression or control-flow-graph context.
///
/// - tags[0]: name of the node
/// - tags[1..]: additional info on the node, e.g. field name in struct expression
/// - args[0]: stmt.id for statements, instr sequence number for instructions
#[derive(Clone)]
#[pyclass(extends = CContextDictionaryRecord, frozen, subclass)]
pub struct CContextNode {}

#[pymethods]
impl CContextNode {
    #[new]
    pub fn new(cxd: Py<PyAny>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CContextDictionaryRecord::new(cxd, ixval))
            .add_subclass(CContextNode {})
    }

    #[getter]
    fn name(slf: PyRef<Self>) -> PyResult<String> {
        slf.into_super()
            .into_super()
            .tags()
            .iter()
            .next()
            .cloned()
            .ok_or_else(|| PyException::new_err("missing"))
    }

    #[getter]
    fn data_id(slf: Py<Self>, py: Python) -> PyResult<isize> {
        let binding = slf.borrow(py).into_super().into_super();
        if let Some(arg0) = binding.args().iter().next() {
            Ok(*arg0)
        } else {
            let name = CContextNode::name(slf.borrow(py))?;
            Err(CHCError::new_err(format!(
                "Context node {name} does not have a data-id"
            )))
        }
    }

    #[pyo3(name = "__str__")]
    pub fn str(slf: PyRef<Self>) -> String {
        let it = slf.into_super().into_super();
        let tags = it.tags().join("_");
        if it.args().is_empty() {
            tags
        } else {
            format!("{tags}:{}", it.args().iter().join("_"))
        }
    }
}

/// Control-flow-graph context expressed by a list of context nodes.
///
/// args[0..]: indices of context nodes in the context dictionary, inner context last
#[derive(Clone)]
#[pyclass(extends = CContextDictionaryRecord, frozen, subclass)]
pub struct CfgContext {}

#[pymethods]
impl CfgContext {
    #[new]
    pub fn new(cxd: Py<PyAny>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CContextDictionaryRecord::new(cxd, ixval))
            .add_subclass(CfgContext {})
    }

    #[getter]
    fn nodes(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let py_super = slf.into_super();
        let cxd = py_super.cxd();
        py_super
            .into_super()
            .args()
            .into_iter()
            .map(|arg| cxd.call_method1(py, intern!(py, "get_node"), (*arg,)))
            .collect()
    }

    #[getter]
    fn reverse_repr(slf: PyRef<Self>, py: Python) -> PyResult<String> {
        Ok(CfgContext::nodes(slf, py)?.into_iter().rev().join("_"))
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>, py: Python) -> PyResult<String> {
        Ok(CfgContext::nodes(slf, py)?.into_iter().join("_"))
    }
}

/// Expression nesting context expressed by a list of context nodes.
///
/// args[0..]: indices of context nodes in the context dictionary, inner context last
#[derive(Clone)]
#[pyclass(extends = CContextDictionaryRecord, frozen, subclass)]
pub struct ExpContext {}

#[pymethods]
impl ExpContext {
    #[new]
    pub fn new(cxd: Py<PyAny>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CContextDictionaryRecord::new(cxd, ixval))
            .add_subclass(ExpContext {})
    }

    #[getter]
    fn nodes(slf: PyRef<Self>, py: Python) -> PyResult<Vec<PyObject>> {
        let py_super = slf.into_super();
        let cxd = py_super.cxd();
        py_super
            .into_super()
            .args()
            .into_iter()
            .map(|arg| cxd.call_method1(py, intern!(py, "get_node"), (*arg,)))
            .collect()
    }

    #[pyo3(name = "__str__")]
    fn str(slf: PyRef<Self>, py: Python) -> PyResult<String> {
        Ok(ExpContext::nodes(slf, py)?.into_iter().join("_"))
    }
}

/// Precise structural placement within a function (relative to ast, exps).
///
/// args[0]: index of cfg context in context dictionary
/// args[1]: index of exp context in context dictionary
#[derive(Clone)]
#[pyclass(extends = CContextDictionaryRecord, frozen, subclass)]
pub struct ProgramContext {}

#[pymethods]
impl ProgramContext {
    #[new]
    pub fn new(cxd: Py<PyAny>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        PyClassInitializer::from(CContextDictionaryRecord::new(cxd, ixval))
            .add_subclass(ProgramContext {})
    }

    #[getter]
    fn cfg_context(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let py_super = slf.into_super();
        let cxd = py_super.cxd();
        let base = py_super.into_super();
        let Some(arg0) = base.args().get(0) else {
            return Err(PyException::new_err("No element 0"));
        };
        cxd.call_method1(py, intern!(py, "get_cfg_context"), (*arg0,))
    }

    #[getter]
    fn exp_context(slf: PyRef<Self>, py: Python) -> PyResult<PyObject> {
        let py_super = slf.into_super();
        let cxd = py_super.cxd();
        let base = py_super.into_super();
        let Some(arg1) = base.args().get(1) else {
            return Err(PyException::new_err("No element 1"));
        };
        cxd.call_method1(py, intern!(py, "get_exp_context"), (*arg1,))
    }

    #[pyo3(name = "__str__")]
    fn str(slf: Py<Self>, py: Python) -> PyResult<String> {
        let cfg_context = ProgramContext::cfg_context(slf.borrow(py), py)?;
        let exp_context = ProgramContext::exp_context(slf.borrow(py), py)?;
        Ok(format!("({cfg_context},{exp_context})"))
    }
}
