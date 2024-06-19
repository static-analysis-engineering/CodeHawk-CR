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

import xml.etree.ElementTree as ET

import chc.util.fileutil as UF
import chc_rust

from typing import Callable, Dict, List, Generic, Optional, Tuple, TypeVar


class IndexedTableError(UF.CHCError):
    def __init__(
            self,
            msg: str,
            itemlist: List[Tuple[int, "IndexedTableValue"]] = []) -> None:
        UF.CHCError.__init__(self, msg)
        self._itemlist = itemlist

    @property
    def itemlist(self) -> List[Tuple[int, "IndexedTableValue"]]:
        return self._itemlist

    def __str__(self) -> str:
        lines: List[str] = []
        if len(self.itemlist) > 0 and len(self.itemlist) < 20:
            lines.append("-")
            for (index, i) in self.itemlist:
                lines.append(str(index).rjust(3) + ": " + str(i))
            lines.append("-")
        lines.append(self.msg)
        return "\n".join(lines)


class IndexedTableValueMismatchError(UF.CHCError):

    def __init__(
            self,
            tag: str,
            reqtagcount: int,
            reqargcount: int,
            acttagcount: int,
            actargcount: int,
            name: str) -> None:
        UF.CHCError.__init__(
            self,
            "Dictionary record mismatch for "
            + tag
            + " in "
            + name
            + ": Expected "
            + str(reqtagcount)
            + " tags and "
            + str(reqargcount)
            + " args, but found "
            + str(acttagcount)
            + " tags and "
            + str(actargcount))


def get_attribute_int_list(node: ET.Element, attr: str) -> List[int]:
    """Return list of integers in attr if attr is present or [] otherwise."""

    xattr = node.get(attr)
    if xattr is None or xattr == "":
        return []
    else:
        return [int(x) for x in xattr.split(",")]


get_rep = chc_rust.util.indexed_table.get_rep


get_key = chc_rust.util.indexed_table.get_key


IndexedTableValue = chc_rust.util.indexed_table.IndexedTableValue


get_value = chc_rust.util.indexed_table.get_value


IndexedTable = chc_rust.util.indexed_table.IndexedTable
