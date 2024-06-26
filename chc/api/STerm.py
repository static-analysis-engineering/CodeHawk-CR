# ------------------------------------------------------------------------------
# CodeHawk C Analyzer
# Author: Henny Sipma
# ------------------------------------------------------------------------------
# The MIT License (MIT)
#
# Copyright (c) 2017-2020 Kestrel Technology LLC
# Copyright (c) 2020-2023 Henny B. Sipma
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
"""Object representation of sum type s_term (term in an external predicate).

Object representation of the corresponding OCaml sumtype: s_term_t: a term that
solely refers to entities that are visible outside of a function, such as its
parameters or return value.

The properties of a term are constructed from its indexed value in the
InterfaceDictionary.

"""
from typing import Dict, List, Optional, TYPE_CHECKING
import xml.etree.ElementTree as ET

from chc.api.InterfaceDictionaryRecord import (
    InterfaceDictionaryRecord, ifdregistry)

import chc.util.fileutil as UF
import chc.util.IndexedTable as IT

if TYPE_CHECKING:
    from chc.api.ApiParameter import ApiParameter
    from chc.api.InterfaceDictionary import InterfaceDictionary
    from chc.api.SOffset import SOffset


printops: Dict[str, str] = {
    "plus": "+",
    "plusa": "+",
    "minnus": "-",
    "times": "*",
    "mult": "*",
    "divide": "/",
    "div": "/",
}


def get_printop(s: str) -> str:
    if s in printops:
        return printops[s]
    else:
        return s


class STerm(InterfaceDictionaryRecord):
    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STerm":
        return super().__new__(cls, ifd, ixval)

    def get_iterm(self, argix: int) -> "STerm":
        return self.ifd.get_s_term(int(self.args[argix]))

    @property
    def is_arg_value(self) -> bool:
        return False

    @property
    def is_local_var(self) -> bool:
        return False

    @property
    def is_return_value(self) -> bool:
        return False

    @property
    def is_named_constant(self) -> bool:
        return False

    @property
    def is_num_constant(self) -> bool:
        return False

    @property
    def is_index_size(self) -> bool:
        return False

    @property
    def is_byte_size(self) -> bool:
        return False

    '''
    def is_field_offset(self) -> bool:
        return False
    '''

    @property
    def is_arg_addressed_value(self) -> bool:
        return False

    @property
    def is_arg_null_terminator_pos(self) -> bool:
        return False

    @property
    def is_arg_size_of_type(self) -> bool:
        return False

    @property
    def is_arithmetic_expr(self) -> bool:
        return False

    @property
    def is_formatted_output_size(self) -> bool:
        return False

    @property
    def is_region(self) -> bool:
        return False

    @property
    def is_runtime_value(self) -> bool:
        return False

    @property
    def is_choice_value(self) -> bool:
        return False

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        return ET.Element("s-term")

    def pretty(self) -> str:
        return self.__str__()

    def __str__(self) -> str:
        return "s-term-" + self.tags[0]


@ifdregistry.register_tag("av", STerm)
class STArgValue(STerm):
    """Argument value passed to a function.

    * args[0]: index of api parameter in interface dictionary
    * args[1]: index of s_term offset in interface dictionary
    """

    def __new__(
        cls, cd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STArgValue":
        return super().__new__(cls, cd, ixval)

    @property
    def parameter(self) -> "ApiParameter":
        return self.ifd.get_api_parameter(int(self.args[0]))

    @property
    def offset(self) -> "SOffset":
        return self.ifd.get_s_offset(int(self.args[1]))

    def is_arg_value(self) -> bool:
        return True

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        node = ET.Element("ci")
        node.text = signature[self.args[0]]
        return node

    def __str__(self) -> str:
        return "arg-val(" + str(self.parameter) + ")"


@ifdregistry.register_tag("lv", STerm)
class STLocalVariable(STerm):
    """Local variable used in external predicate.

    * tags[1]: name
"""

    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STLocalVariable":
        return super().__new__(cls, ifd, ixval)

    @property
    def is_local_var(self) -> bool:
        return True

    @property
    def name(self) -> str:
        return self.tags[1]

    def __str__(self) -> str:
        return self.name


@ifdregistry.register_tag("rv", STerm)
class STReturnValue(STerm):
    """Return value, as used in post conditions."""

    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STReturnValue":
        return super().__new__(cls, ifd, ixval)

    @property
    def is_return_value(self) -> bool:
        return True

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        return ET.Element("return")

    def __str__(self) -> str:
        return "returnval"


@ifdregistry.register_tag("nc", STerm)
class STNamedConstant(STerm):
    """Named constant with unspecified value.

    * tags[1]: name
    """

    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STNamedConstant":
        return super().__new__(cls, ifd, ixval)

    @property
    def name(self) -> str:
        return self.tags[1]

    @property
    def is_named_constant(self) -> bool:
        return True

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        node = ET.Element("ci")
        node.text = self.name
        return node

    def __str__(self) -> str:
        return "named-constant(" + self.name + ")"


@ifdregistry.register_tag("ic", STerm)
class STNumConstant(STerm):
    """Constant with given numerical value."""

    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STNumConstant":
        return super().__new__(cls, ifd, ixval)

    @property
    def constantvalue(self) -> int:
        try:
            return int(self.tags[1])
        except ValueError as e:
            raise UF.CHCError(str(e))

    @property
    def is_num_constant(self) -> bool:
        return True

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        node = ET.Element("cn")
        node.text = str(self.constantvalue)
        return node

    def pretty(self) -> str:
        return str(self.constantvalue)

    def __str__(self) -> str:
        return "num-constant(" + str(self.constantvalue) + ")"


@ifdregistry.register_tag("is", STerm)
class STIndexSize(STerm):
    """Size term expressed in units of array index.

    For example, an index-size of 1 corresponds to 4 bytes in an int-array.

    * args[0]: index of term in interface dictionary
    """

    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STIndexSize":
        return super().__new__(cls, ifd, ixval)

    @property
    def term(self) -> STerm:
        return self.get_iterm(0)

    @property
    def is_index_size(self) -> bool:
        return True

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        anode = ET.Element("apply")
        opnode = ET.Element("index-size")
        tnode = self.term.get_mathml_node(signature)
        anode.extend([opnode, tnode])
        return anode

    def __str__(self) -> str:
        return "index-size(" + str(self.term) + ")"


@ifdregistry.register_tag("bs", STerm)
class STByteSize(STerm):
    """Size term expressed in bytes.

    * args[0]: index of term in interface dictionary
    """

    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STByteSize":
        return super().__new__(cls, ifd, ixval)

    @property
    def term(self) -> STerm:
        return self.get_iterm(0)

    @property
    def is_byte_size(self) -> bool:
        return True

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        anode = ET.Element("apply")
        opnode = ET.Element("byte-size")
        tnode = self.term.get_mathml_node(signature)
        anode.extend([opnode, tnode])
        return anode

    def __str__(self) -> str:
        return "byte-size(" + str(self.term) + ")"


'''
@ifdregistry.register_tag("fo", STerm)
class STFieldOffset(STerm):
    def __init__(
        self,
        cd: "InterfaceDictionary",
        ixval: IT.IndexedTableValue,
    ) -> None:
        STerm.__init__(self, cd, ixval)

    def get_name(self) -> str:
        return self.tags[1]

    def is_field_offset(self) -> bool:
        return True

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        node = ET.Element("field")
        node.set("fname", self.get_name())
        return node

    def __str__(self) -> str:
        return "field-offset(" + str(self.get_name()) + ")"
'''


@ifdregistry.register_tag("aa", STerm)
class STArgAddressedValue(STerm):
    """The value addressed by an argument.

    * args[0]: index of term in interface dictionary
    * args[1]: index of term offset in interface dictionary
    """

    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STArgAddressedValue":
        return super().__new__(cls, ifd, ixval)

    @property
    def term(self) -> STerm:
        return self.get_iterm(0)

    @property
    def offset(self) -> "SOffset":
        return self.ifd.get_s_offset(int(self.args[1]))

    @property
    def is_arg_addressed_value(self) -> bool:
        return True

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        anode = ET.Element("apply")
        opnode = ET.Element("addressed-value")
        t1node = self.term.get_mathml_node(signature)
        offnode = self.offset.get_mathml_node(signature)
        if offnode is not None:
            t1node.append(offnode)
        anode.extend([opnode, t1node])
        return anode

    def __str__(self) -> str:
        return (
            "addressed-value("
            + str(self.term)
            + ")"
            + str(self.offset)
        )


@ifdregistry.register_tag("at", STerm)
class STArgNullTerminatorPos(STerm):
    """The position of the null-terminator in a string term.

    * args[0]: index of term in interface dictionary
    """

    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STArgNullTerminatorPos":
        return super().__new__(cls, ifd, ixval)

    @property
    def term(self) -> STerm:
        return self.get_iterm(0)

    @property
    def is_arg_null_terminator_pos(self) -> bool:
        return True

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        anode = ET.Element("apply")
        opnode = ET.Element("nullterminator-pos")
        tnode = self.term.get_mathml_node(signature)
        anode.extend([opnode, tnode])
        return anode

    def __str__(self) -> str:
        return "arg-null-terminator-pos(" + str(self.term) + ")"


@ifdregistry.register_tag("st", STerm)
class STArgSizeOfType(STerm):
    """Size of argument type, in bytes.

    * args[0]: index of term in interface dictionary
    """

    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STArgSizeOfType":
        return super().__new__(cls, ifd, ixval)

    @property
    def term(self) -> STerm:
        return self.get_iterm(0)

    @property
    def is_arg_size_of_type(self) -> bool:
        return True

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        anode = ET.Element("apply")
        opnode = ET.Element("size-of-type")
        tnode = self.term.get_mathml_node(signature)
        anode.extend([opnode, tnode])
        return anode

    def __str__(self) -> str:
        return "arg-size-of-type(" + str(self.term) + ")"


@ifdregistry.register_tag("ax", STerm)
class STArithmeticExpr(STerm):
    """Binary arithmetic expression on terms.

    * tags[1]: binary operator
    * args[0]: index of first term in interface dictionary
    * args[1]: index of second term in interface dictionary
    """

    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STArithmeticExpr":
        return super().__new__(cls, ifd, ixval)

    @property
    def op(self) -> str:
        return self.tags[1]

    @property
    def term1(self) -> STerm:
        return self.get_iterm(0)

    @property
    def term2(self) -> STerm:
        return self.get_iterm(1)

    @property
    def is_arithmetic_expr(self) -> bool:
        return True

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        anode = ET.Element("apply")
        opnode = ET.Element(self.op)
        t1node = self.term1.get_mathml_node(signature)
        t2node = self.term2.get_mathml_node(signature)
        anode.extend([opnode, t1node, t2node])
        return anode

    def pretty(self) -> str:
        return (
            "("
            + self.term1.pretty()
            + " "
            + get_printop(self.op)
            + " "
            + self.term2.pretty()
            + ")"
        )

    def __str__(self) -> str:
        return (
            "xpr("
            + str(self.term1)
            + " "
            + self.op
            + " "
            + str(self.term2)
            + ")"
        )


@ifdregistry.register_tag("fs", STerm)
class STFormattedOutputSize(STerm):
    """The size of a string formed via a format string.

    * args[0]: index of term in interface dictionary
    """

    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STFormattedOutputSize":
        return super().__new__(cls, ifd, ixval)

    @property
    def term(self) -> STerm:
        return self.get_iterm(0)

    @property
    def is_formatted_output_size(self) -> bool:
        return True

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        anode = ET.Element("apply")
        opnode = ET.Element("formatted-output-size")
        tnode = self.term.get_mathml_node(signature)
        anode.extend([opnode, tnode])
        return anode

    def __str__(self) -> str:
        return "formatted-output-size(" + str(self.term) + ")"


@ifdregistry.register_tag("rg", STerm)
class STRegion(STerm):
    """A memory region.

    * args[0]: index of term in interface dictionary
    """

    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STRegion":
        return super().__new__(cls, ifd, ixval)

    @property
    def term(self) -> STerm:
        return self.get_iterm(0)

    @property
    def is_region(self) -> bool:
        return True

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        anode = ET.Element("apply")
        opnode = ET.Element("region")
        tnode = self.term.get_mathml_node(signature)
        anode.extend([opnode, tnode])
        return anode

    def __str__(self) -> str:
        return "region(" + str(self.term) + ")"


@ifdregistry.register_tag("rt", STerm)
class STRuntimeValue(STerm):
    """A value that is determined at runtime."""

    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STRuntimeValue":
        return super().__new__(cls, ifd, ixval)

    @property
    def is_runtime_value(self) -> bool:
        return True

    def get_mathml_node(self, signature: List[str]) -> ET.Element:
        return ET.Element("runtime-value")

    def __str__(self) -> str:
        return "runtime-value"


@ifdregistry.register_tag("cv", STerm)
class STChoiceValue(STerm):
    """Any value between an optional lowerbound and upperbound.

    * args[0]: index of optional lower bound in interface dictionary
    * args[1]: index of optional upper bound in interface dictionary
    """

    def __new__(
        cls, ifd: "InterfaceDictionary", ixval: IT.IndexedTableValue
    ) -> "STChoiceValue":
        return super().__new__(cls, ifd, ixval)

    @property
    def termlb(self) -> Optional[STerm]:
        return self.ifd.get_opt_s_term(self.args[0])

    @property
    def termub(self) -> Optional[STerm]:
        return self.ifd.get_opt_s_term(self.args[1])

    @property
    def is_choice_value(self) -> bool:
        return True

    def __str__(self) -> str:
        lb = str(self.termlb) if self.termlb is not None else ".."
        ub = str(self.termub) if self.termub is not None else ".."
        return f"[{lb} ; {ub}]"
