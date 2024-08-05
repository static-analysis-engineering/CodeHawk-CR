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
//! Variable definition.

use pyo3::{intern, prelude::*};

use crate::{
    app::{
        c_declarations::CDeclarations, c_dictionary_record::CDeclarationsRecord,
        c_init_info::CInitInfo, c_location::CLocation, c_typ::CTyp,
    },
    util::indexed_table::IndexedTableValue,
};

pub fn module(py: Python) -> PyResult<Bound<PyModule>> {
    let module = PyModule::new_bound(py, "c_var_info")?;
    module.add_class::<CVarInfo>()?;
    Ok(module)
}

/// Local or global variable.
///
/// * tags[0]: vname
/// * tags[1]: vstorage  ('?' for global variable, 'o_gvid' for opaque variable)
///
/// * args[0]: vid       (-1 for global variable)
/// * args[1]: vtype
/// * args[2]: vattr     (-1 for global variable) (TODO: add global attributes)
/// * args[3]: vglob
/// * args[4]: vinline
/// * args[5]: vdecl     (-1 for global variable) (TODO: add global locations)
/// * args[6]: vaddrof
/// * args[7]: vparam
/// * args[8]: vinit     (optional)
#[pyclass(extends = CDeclarationsRecord, frozen, subclass)]
pub struct CVarInfo {
    #[pyo3(get)]
    vname: String,
    #[pyo3(get)]
    real_vid: isize,
    vtype: isize,
    #[pyo3(get)]
    vglob: bool,
    #[pyo3(get)]
    vinline: bool,
    vdecl: isize,
    #[pyo3(get)]
    vaddrof: bool,
    #[pyo3(get)]
    vparam: isize,
    vinit: Option<isize>,
}

#[pymethods]
impl CVarInfo {
    #[new]
    fn new(cd: Py<CDeclarations>, ixval: IndexedTableValue) -> PyClassInitializer<Self> {
        let var_info = CVarInfo {
            vname: ixval.tags()[0].clone(),
            real_vid: ixval.args()[0],
            vtype: ixval.args()[1],
            vglob: ixval.args()[3] == 1,
            vinline: ixval.args()[4] == 1,
            vdecl: ixval.args()[5],
            vaddrof: ixval.args()[6] == 1,
            vparam: ixval.args()[7],
            vinit: ixval.args().get(8).cloned(),
        };
        PyClassInitializer::from(CDeclarationsRecord::new(cd, ixval)).add_subclass(var_info)
    }

    #[getter]
    fn vtype<'a>(slf: &Bound<'a, Self>) -> PyResult<Bound<'a, CTyp>> {
        Ok(slf
            .borrow()
            .into_super()
            .dictionary(slf.py())?
            .call_method1(intern!(slf.py(), "get_typ"), (slf.get().vtype,))?
            .downcast()?
            .clone())
    }

    #[getter]
    fn is_global(&self) -> bool {
        self.vglob
    }

    #[getter]
    fn is_inline(&self) -> bool {
        self.vinline
    }

    #[getter]
    fn vdecl<'a>(slf: &Bound<'a, Self>) -> PyResult<Option<Bound<'a, CLocation>>> {
        let vdecl = slf.get().vdecl;
        Ok(if vdecl < 0 {
            None
        } else {
            Some(
                slf.borrow()
                    .into_super()
                    .decls()
                    .bind(slf.py())
                    .call_method1(intern!(slf.py(), "get_location"), (vdecl,))?
                    .downcast()?
                    .clone(),
            )
        })
    }

    #[getter]
    fn is_param(&self) -> bool {
        self.vparam > 0
    }

    #[getter]
    fn vinit<'a>(slf: &Bound<'a, Self>) -> PyResult<Option<Bound<'a, CInitInfo>>> {
        let Some(vinit) = slf.get().vinit else {
            return Ok(None);
        };
        Ok(Some(
            slf.borrow()
                .into_super()
                .decls()
                .bind(slf.py())
                .call_method1(intern!(slf.py(), "get_initinfo"), (vinit,))?
                .downcast()?
                .clone(),
        ))
    }

    #[getter]
    fn vid(slf: PyRef<Self>) -> isize {
        if slf.real_vid >= 0 {
            slf.real_vid
        } else {
            slf.into_super().into_super().index()
        }
    }
}
