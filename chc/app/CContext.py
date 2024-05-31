# ------------------------------------------------------------------------------
# CodeHawk C Analyzer
# Author: Henny Sipma
# ------------------------------------------------------------------------------
# The MIT License (MIT)
#
# Copyright (c) 2017-2020 Kestrel Technology LLC
# Copyright (c) 2021-2022 Henny B. Sipma
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
"""Structural context description to locate a program entity.


"""

from typing import List, TYPE_CHECKING

import chc.util.fileutil as UF
import chc_rust
from chc.util.IndexedTable import IndexedTableValue

if TYPE_CHECKING:
    from chc.app.CContextDictionary import CContextDictionary


CContextDictionaryRecord = chc_rust.app.c_context.CContextDictionaryRecord


CContextNode = chc_rust.app.c_context.CContextNode


CfgContext = chc_rust.app.c_context.CfgContext


ExpContext = chc_rust.app.c_context.ExpContext


class ProgramContext(CContextDictionaryRecord):
    """Precise structural placement within a function (relative to ast, exps).

    args[0]: index of cfg context in context dictionary
    args[1]: index of exp context in context dictionary
    """

    def __new__(
            cls, cxd: "CContextDictionary", ixval: IndexedTableValue
    ) -> "ProgramContext":
        return super().__new__(cls, cxd, ixval)

    @property
    def cfg_context(self) -> CfgContext:
        return self.cxd.get_cfg_context(self.args[0])

    @property
    def exp_context(self) -> ExpContext:
        return self.cxd.get_exp_context(self.args[1])

    def __str__(self) -> str:
        return "(" + str(self.cfg_context) + "," + str(self.exp_context) + ")"
