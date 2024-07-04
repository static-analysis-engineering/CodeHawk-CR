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
"""Object representation of CIL attrparam sum type."""

from typing import List, Tuple, TYPE_CHECKING

from chc.app.CDictionaryRecord import CDictionaryRecord, cdregistry

import chc.util.fileutil as UF
import chc.util.IndexedTable as IT
from chc.util.loggingutil import chklogger

import chc_rust

if TYPE_CHECKING:
    from chc.app.CDictionary import CDictionary
    from chc.app.CTyp import CTyp
    from chc.app.CTypsig import CTypsig


CAttr = chc_rust.app.c_attributes.CAttr


CAttrInt = chc_rust.app.c_attributes.CAttrInt


CAttrStr = chc_rust.app.c_attributes.CAttrStr


CAttrCons = chc_rust.app.c_attributes.CAttrCons


@cdregistry.register_tag("asizeof", CAttr)
class CAttrSizeOf(CAttr):
    """Attribute that describes the size of a type.

    * args[0]: index of target type in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttrSizeOf":
        return super().__new__(cls, cd, ixval)

    @property
    def typ(self) -> "CTyp":
        return self.cd.get_typ(int(self.args[0]))

    @property
    def is_sizeof(self) -> bool:
        return True

    def __str__(self) -> str:
        return "asizeof(" + str(self.typ) + ")"


@cdregistry.register_tag("asizeofe", CAttr)
class CAttrSizeOfE(CAttr):
    """Size of an attribute parameter.

    * args[0]: index of argument parameter in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttrSizeOfE":
        return super().__new__(cls, cd, ixval)

    @property
    def param(self) -> CAttr:
        return self.cd.get_attrparam(int(self.args[0]))

    @property
    def is_sizeofe(self) -> bool:
        return True

    def __str__(self) -> str:
        return "asizeofe(" + str(self.param) + ")"


@cdregistry.register_tag("asizeofs", CAttr)
class CAttrSizeOfS(CAttr):
    """Replacement ASizeOf in type signatures.

    * args[0]: index of target typsig in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttrSizeOfS":
        return super().__new__(cls, cd, ixval)

    @property
    def typsig(self) -> "CTypsig":
        return self.cd.get_typsig(int(self.args[0]))

    @property
    def is_sizeofs(self) -> bool:
        return True

    def __str__(self) -> str:
        return "asizeofs(" + str(self.typsig) + ")"


@cdregistry.register_tag("aalignof", CAttr)
class CAttrAlignOf(CAttr):
    """Alignment of a type.

    * args[0]: index of target type in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttrAlignOf":
        return super().__new__(cls, cd, ixval)

    @property
    def typ(self) -> "CTyp":
        return self.cd.get_typ(int(self.args[0]))

    @property
    def is_alignof(self) -> bool:
        return True

    def __str__(self) -> str:
        return "aalignof(" + str(self.typ) + ")"


@cdregistry.register_tag("aalignofe", CAttr)
class CAttrAlignOfE(CAttr):
    """Alignment of an attribute parameter.

    * args[0]: index of attribute parameter in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttrAlignOfE":
        return super().__new__(cls, cd, ixval)

    @property
    def param(self) -> CAttr:
        return self.cd.get_attrparam(int(self.args[0]))

    @property
    def is_alignofe(self) -> bool:
        return True

    def __str__(self) -> str:
        return "aalignofe(" + str(self.param) + ")"


@cdregistry.register_tag("aalignofs", CAttr)
class CAttrAlignOfS(CAttr):
    """Alignment of a type signature.

    * args[0]: target type signature
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttrAlignOfS":
        return super().__new__(cls, cd, ixval)

    @property
    def typsig(self) -> "CTypsig":
        return self.cd.get_typsig(int(self.args[0]))

    @property
    def is_alignofs(self) -> bool:
        return True

    def __str__(self) -> str:
        return "aalignofs(" + str(self.typsig) + ")"


@cdregistry.register_tag("aunop", CAttr)
class CAttrUnOp(CAttr):
    """Unary attribute parameter operation.

    * tags[1]: operator
    * args[0]: index of attribute parameter in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttrUnOp":
        return super().__new__(cls, cd, ixval)

    @property
    def op(self) -> str:
        return self.tags[1]

    @property
    def param(self) -> CAttr:
        return self.cd.get_attrparam(int(self.args[0]))

    @property
    def is_unop(self) -> bool:
        return True

    def __str__(self) -> str:
        return "aunop(" + self.op + "," + str(self.param) + ")"


@cdregistry.register_tag("abinop", CAttr)
class CAttrBinOp(CAttr):
    """Binary attribute parameter operation.

    * tags[1]: operator
    * args[0]: index of first attribute parameter in cdictionary
    * args[1]: index of second attribute parameter in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttrBinOp":
        return super().__new__(cls, cd, ixval)

    @property
    def op(self) -> str:
        return self.tags[1]

    @property
    def param1(self) -> CAttr:
        return self.cd.get_attrparam(int(self.args[0]))

    @property
    def param2(self) -> CAttr:
        return self.cd.get_attrparam(int(self.args[1]))

    @property
    def is_binop(self) -> bool:
        return True

    def __str__(self) -> str:
        return (
            "abinop("
            + str(self.param1)
            + " "
            + self.op
            + " "
            + str(self.param2)
            + ")"
        )


@cdregistry.register_tag("adot", CAttr)
class CAttrDot(CAttr):
    """Dot operator on attributes.

    * tags[1]: string suffix
    * args[0]: index of attribute parameter in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttrDot":
        return super().__new__(cls, cd, ixval)

    @property
    def suffix(self) -> str:
        return self.tags[1]

    @property
    def param(self) -> CAttr:
        return self.cd.get_attrparam(int(self.args[0]))

    @property
    def is_dot(self) -> bool:
        return True

    def __str__(self) -> str:
        return "adot(" + str(self.param) + "." + self.suffix + ")"


@cdregistry.register_tag("astr", CAttr)
class CAttrStar(CAttr):
    """Star operation on attribute.

    * args[0]: index of attribute parameter in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttrStar":
        return super().__new__(cls, cd, ixval)

    @property
    def param(self) -> CAttr:
        chklogger.logger.info("Self-referential attribute (*)")
        if self.index == int(self.args[0]):
            return self
        else:
            return self.cd.get_attrparam(int(self.args[0]))

    @property
    def is_star(self) -> bool:
        return True

    def __str__(self) -> str:
        if self.index == self.args[0]:
            return "astar()"
        else:
            return "astar(" + str(self.param) + ")"


@cdregistry.register_tag("aaddrof", CAttr)
class CAttrAddrOf(CAttr):
    """Addressof operator on attribute.

    * args[0]: index of attribute parameter in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttrAddrOf":
        return super().__new__(cls, cd, ixval)

    @property
    def param(self) -> CAttr:
        return self.cd.get_attrparam(int(self.args[0]))

    @property
    def is_addrof(self) -> bool:
        return True

    def __str__(self) -> str:
        return "aaddrof(" + str(self.param) + ")"


@cdregistry.register_tag("aindex", CAttr)
class CAttrIndex(CAttr):
    """Index operation on attributes

    * args[0]: index of first attribute parameter in cdictionary
    * args[1]: index of second attribute parameter in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttrIndex":
        return super().__new__(cls, cd, ixval)

    @property
    def param1(self) -> CAttr:
        return self.cd.get_attrparam(int(self.args[0]))

    @property
    def param2(self) -> CAttr:
        return self.cd.get_attrparam(int(self.args[1]))

    @property
    def is_index(self) -> bool:
        return True

    def __str__(self) -> str:
        return "aindex(" + str(self.param1) + "," + str(self.param2) + ")"


@cdregistry.register_tag("aquestion", CAttr)
class CAttrQuestion(CAttr):
    """Question operator on attributes

    * args[0]: index of first attribute paramter in cdictionary
    * args[1]: index of second attribute parameter in cdictionary
    * args[2]: index of third attribute parameter in cdictionary
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttrQuestion":
        return super().__new__(cls, cd, ixval)

    @property
    def param1(self) -> CAttr:
        return self.cd.get_attrparam(int(self.args[0]))

    @property
    def param2(self) -> CAttr:
        return self.cd.get_attrparam(int(self.args[1]))

    @property
    def param3(self) -> CAttr:
        return self.cd.get_attrparam(int(self.args[2]))

    def __str__(self) -> str:
        return (
            "aquestion("
            + str(self.param1)
            + ","
            + str(self.param2)
            + ","
            + str(self.param3)
            + ")"
        )


class CAttribute(CDictionaryRecord):

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttribute":
        return super().__new__(cls, cd, ixval)

    @property
    def name(self) -> str:
        return self.tags[0]

    @property
    def params(self) -> List[CAttr]:
        return [self.cd.get_attrparam(int(i)) for i in self.args]

    def __str__(self) -> str:
        return self.name + ": " + ",".join([str(p) for p in self.params])


class CAttributes(chc_rust.app.c_attributes.CAttributes):

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CAttributes":
        return super().__new__(cls, cd, ixval)

    @property
    def attributes(self) -> List[CAttribute]:
        return [self.cd.get_attribute(int(i)) for i in self.args]

    @property
    def length(self) -> int:
        return len(self.attributes)

    def __str__(self) -> str:
        return ",".join([str(p) for p in self.attributes])
