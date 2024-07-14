# ------------------------------------------------------------------------------
# CodeHawk C Analyzer
# Author: Henny Sipma
# ------------------------------------------------------------------------------
# The MIT License (MIT)
#
# Copyright (c) 2017-2020 Kestrel Technology LLC
# Copyright (c) 2020-2022 Henny B. Sipma
# Copyright (c) 2023-2024 Aarno Labs LLC
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
# ------------------------------------------------------------------------------
"""Variant type for the CIL **typ** data type."""

import xml.etree.ElementTree as ET

from typing import Any, cast, Dict, List, Optional, Tuple, TYPE_CHECKING

from chc.app.CDictionaryRecord import CDictionaryRecord, cdregistry

import chc.util.fileutil as UF
import chc.util.IndexedTable as IT
from chc.util.loggingutil import chklogger

import chc_rust

if TYPE_CHECKING:
    from chc.app.CDictionary import CDictionary
    from chc.app.CExp import CExp, CExpConst
    from chc.app.CAttributes import CAttributes
    from chc.app.CCompInfo import CCompInfo
    from chc.app.CConst import CConst, CConstInt


CTyp = chc_rust.app.c_typ.CTyp


CTypVoid = chc_rust.app.c_typ.CTypVoid


CTypInt = chc_rust.app.c_typ.CTypInt


CTypFloat = chc_rust.app.c_typ.CTypFloat


CTypNamed = chc_rust.app.c_typ.CTypNamed


CTypComp = chc_rust.app.c_typ.CTypComp


CTypEnum = chc_rust.app.c_typ.CTypEnum


CTypBuiltinVaargs = chc_rust.app.c_typ.CTypBuiltinVaargs


CTypPtr = chc_rust.app.c_typ.CTypPtr


@cdregistry.register_tag("tarray", CTyp)
class CTypArray(CTyp):
    """ Array type

    * args[0]: index of base type in cdictionary
    * args[1]: index of size expression in cdictionary (optional)
    * args[2]: index of attributes in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CTypArray":
        return super().__new__(cls, cd, ixval)

    @property
    def array_basetype(self) -> CTyp:
        return self.get_typ(self.args[0])

    @property
    def array_size_expr(self) -> "CExp":
        if self.args[1] >= 0:
            return self.get_exp(self.args[1])
        else:
            raise UF.CHCError("Array type does not have a size")

    def has_array_size_expr(self) -> bool:
        return self.args[1] >= 0

    @property
    def size(self) -> int:
        try:
            if self.has_array_size_expr():
                array_size_const = cast(
                    "CExpConst", self.array_size_expr).constant
                array_size_int = cast(
                    "CConstInt", array_size_const).intvalue
                return self.array_basetype.size * array_size_int
        except BaseException:
            return -1000
        else:
            return -1000

    @property
    def is_array(self) -> bool:
        return True

    def get_opaque_type(self) -> CTyp:
        tags = ["tvoid"]
        args: List[int] = []
        return self.cd.get_typ(self.cd.mk_typ_index(tags, args))

    def to_dict(self) -> Dict[str, Any]:
        result = {"base": "array", "elem": self.array_basetype.to_dict()}
        if self.has_array_size_expr() and self.array_size_expr.is_constant:
            result["size"] = str(self.array_size_expr)
        return result

    def __str__(self) -> str:
        size = self.array_size_expr
        ssize = str(size) if size is not None else "?"
        return str(self.array_basetype) + "[" + ssize + "]"


@cdregistry.register_tag("tfun", CTyp)
class CTypFun(CTyp):
    """Function type

    * args[0]: index of return type in cdictionary
    * args[1]: index of argument types list in cdictionary (optional
    * args[2]: 1 = varargs
    * args[3]: index of attributes in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CTypFun":
        return super().__new__(cls, cd, ixval)

    @property
    def return_type(self) -> CTyp:
        return self.get_typ(self.args[0])

    @property
    def funargs(self) -> Optional["CFunArgs"]:
        return self.cd.get_funargs_opt(self.args[1])

    @property
    def size(self) -> int:
        return 4

    @property
    def is_function(self) -> bool:
        return True

    def get_opaque_type(self) -> CTyp:
        tags = ["tvoid"]
        args: List[int] = []
        return self.cd.get_typ(self.cd.mk_typ_index(tags, args))

    @property
    def is_default_function_prototype(self) -> bool:
        funargs = self.funargs
        if funargs is None:
            return True
        else:
            args = funargs.arguments
            return len(args) > 0 and all(
                [x.name.startswith("$par$") for x in args]
            )

    @property
    def is_vararg(self) -> bool:
        return self.args[2] == 1

    def strip_attributes(self) -> CTyp:
        rtype = self.return_type.strip_attributes()
        if rtype.index != self.return_type.index:
            newargs = self.args[:]
            newargs[0] = rtype.index
            newtypix = self.cd.mk_typ_index(self.tags, newargs)
            newtyp = self.cd.get_typ(newtypix)
            chklogger.logger.info(
                "Change function type from %s to %s", str(self), str(newtyp))
            return newtyp
        else:
            return self

    def to_dict(self) -> Dict[str, Any]:
        result: Dict[str, Any] = {
            "base": "fun", "rvtype": self.return_type.to_dict()}
        if self.is_default_function_prototype:
            result["default"] = "true"
        elif self.funargs is not None:
            result["args"] = self.funargs.to_dict()
        return result

    def __str__(self) -> str:
        rtyp = self.return_type
        args = self.funargs
        return "(" + str(args) + "):" + str(rtyp)


class CFunArg(CDictionaryRecord):
    """Function argument

    * tags[0]: argument name
    * args[0]: index of argument type in cdictionary
    * args[1]: index of attributes in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CFunArg":
        return super().__new__(cls, cd, ixval)

    @property
    def name(self) -> str:
        if len(self.tags) > 0:
            return self.tags[0]
        else:
            return "__"

    @property
    def typ(self) -> CTyp:
        return self.cd.get_typ(self.args[0])

    def to_dict(self) -> Dict[str, Any]:
        return self.typ.to_dict()

    def __str__(self) -> str:
        return str(self.typ) + " " + self.name


class CFunArgs(CDictionaryRecord):
    """Function arguments

    * args[0..]: indices of function arguments in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CFunArgs":
        return super().__new__(cls, cd, ixval)

    @property
    def arguments(self) -> List[CFunArg]:
        return [self.cd.get_funarg(i) for i in self.args]

    def to_dict(self) -> List[Dict[str, Any]]:
        return [a.to_dict() for a in self.arguments]

    def __str__(self) -> str:
        return "(" + ", ".join([str(x) for x in self.arguments]) + ")"
