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
"""Object representation of CIL constant sum type."""

from typing import List, Tuple, TYPE_CHECKING

from chc.app.CDictionaryRecord import CDictionaryRecord, cdregistry

import chc_rust
import chc.util.IndexedTable as IT

if TYPE_CHECKING:
    from chc.app.CDictionary import CDictionary
    from chc.app.CExp import CExp


CConst = chc_rust.app.c_const.CConst


CConstInt = cdregistry.register_tag("int", CConst)(chc_rust.app.c_const.CConstInt)


CConstStr = cdregistry.register_tag("str", CConst)(chc_rust.app.c_const.CConstStr)


CConstWStr = cdregistry.register_tag("wstr", CConst)(chc_rust.app.c_const.CConstWStr)


CConstChr = cdregistry.register_tag("chr", CConst)(chc_rust.app.c_const.CConstChr)


CConstReal = cdregistry.register_tag("real", CConst)(chc_rust.app.c_const.CConstReal)


CConstReal = cdregistry.register_tag("enum", CConst)(chc_rust.app.c_const.CConstEnum)


class CStringConstant(CDictionaryRecord):
    """Constant string value

    - tags[0]: string value or hexadecimal representation of string value
    - tags[1]: 'x' (optional) if string value is represented in hexadecimal

    - args[0] length of original string
    """

    def __new__(cls, cd: "CDictionary", ixval: IT.IndexedTableValue) -> "CstringConstant":
        return super().__new__(cls, cd, ixval)

    @property
    def stringvalue(self) -> str:
        if len(self.tags) > 0:
            return self.tags[0]
        else:  # empty string is filtered out
            return ""

    @property
    def string_length(self) -> int:
        return self.args[0]

    @property
    def is_hex(self) -> bool:
        return len(self.tags) > 1

    def __str__(self) -> str:
        if self.is_hex:
            return "(" + str(self.string_length) + "-char string"
        else:
            return self.stringvalue
