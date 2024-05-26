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
"""Base value and type of a memory reference."""

from typing import Any, Dict, List, TYPE_CHECKING

from chc.invariants.CFunDictionaryRecord import (
    CFunVarDictionaryRecord, varregistry)

import chc.util.fileutil as UF

from chc.util.IndexedTable import IndexedTableValue

if TYPE_CHECKING:
    from chc.app.CTyp import CTyp
    from chc.invariants.CFunVarDictionary import CFunVarDictionary
    from chc.invariants.CVMemoryBase import CVMemoryBase


class CVMemoryReferenceData(CFunVarDictionaryRecord):

    def __new__(
            cls, vd: "CFunVarDictionary", ixval: IndexedTableValue) -> "CVMemoryReferenceData":
        return super().__new__(cls, vd, ixval)

    @property
    def base(self) -> "CVMemoryBase":
        return self.vd.get_memory_base(self.args[0])

    @property
    def typ(self) -> "CTyp":
        return self.cd.get_typ(self.args[1])

    def __str__(self) -> str:
        return str(self.base)
