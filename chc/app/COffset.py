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
"""Object representation for CIL offset sum type."""

from typing import Dict, List, Tuple, TYPE_CHECKING

from chc.app.CDictionaryRecord import CDictionaryRecord, cdregistry

import chc.util.IndexedTable as IT

import chc_rust

if TYPE_CHECKING:
    from chc.app.CDictionary import CDictionary
    from chc.app.CExp import CExp


COffset = chc_rust.app.c_offset.COffset


CNoOffset = cdregistry.register_tag("n", COffset)(chc_rust.app.c_offset.CNoOffset)


CFieldOffset = cdregistry.register_tag("f", COffset)(chc_rust.app.c_offset.CFieldOffset)


@cdregistry.register_tag("i", COffset)
class CIndexOffset(COffset):
    """Index offset into an array.

    * args[0]: index of base of index expression in cdictionary
    * args[1]: index of sub-offset in cdictionary
    """
    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CIndexOffset":
        return super().__new__(cls, cd, ixval)

    @property
    def index_exp(self) -> "CExp":
        return self.cd.get_exp(self.args[0])

    @property
    def offset(self) -> COffset:
        return self.cd.get_offset(self.args[1])

    def get_strings(self) -> List[str]:
        return self.index_exp.get_strings()

    def get_variable_uses(self, vid: int) -> int:
        return self.index_exp.get_variable_uses(vid)

    @property
    def is_index(self) -> bool:
        return True

    def to_dict(self) -> Dict[str, object]:
        result: Dict[str, object] = {
            "base": "index-offset", "exp": self.index_exp.to_dict()}
        if self.offset.has_offset():
            result["offset"] = self.offset.to_dict()
        return result

    def __str__(self) -> str:
        offset = str(self.offset) if self.has_offset() else ""
        return "[" + str(self.index_exp) + "]" + offset
