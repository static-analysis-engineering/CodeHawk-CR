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


get_key = chc_rust.util.IndexedTable.get_key
IndexedTableSuperclass = chc_rust.util.IndexedTable.IndexedTableSuperclass


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


def get_rep(node: ET.Element) -> Tuple[int, List[str], List[int]]:
    tags = node.get("t")
    args = node.get("a")
    try:
        if tags is None:
            taglist = []
        else:
            taglist = tags.split(",")
        if args is None or args == "":
            arglist = []
        else:
            arglist = [int(x) for x in args.split(",")]
        index_str = node.get("ix")
        if index_str is None:
            raise Exception("node " + str(node) + " did not have an ix element")
        index = int(index_str)
        return (index, taglist, arglist)
    except Exception as e:
        print("tags: " + str(tags))
        print("args: " + str(args))
        print(e)
        raise


IndexedTableValue = chc_rust.util.IndexedTable.IndexedTableValue


def get_value(node: ET.Element) -> IndexedTableValue:
    rep = get_rep(node)
    return IndexedTableValue(*rep)


class IndexedTable(IndexedTableSuperclass):
    """Table to provide unique indices to objects represented by a key string.

    The table can be checkpointed and reset to that checkpoint with
    - set_checkpoint
    - reset_to_checkpoint

    Note: the string encodings use the comma as a concatenation character, hence
          the comma character cannot be used in any string representation.
    """

    def __new__(cls, name: str) -> "IndexedTable":
        return super().__new__(cls, name)

    def retrieve_by_key(
        self, f: Callable[[Tuple[str, str]], bool]
    ) -> List[Tuple[Tuple[str, str], IndexedTableValue]]:
        result: List[Tuple[Tuple[str, str], IndexedTableValue]] = []
        for key in self.keytable:
            if f(key):
                result.append((key, self.indextable[self.keytable[key]]))
        return result

    def read_xml(
        self,
        node: Optional[ET.Element],
        tag: str,
        get_value: Callable[
            [ET.Element], IndexedTableValue] = lambda x: get_value(x),
        get_key: Callable[
            [IndexedTableValue], Tuple[str, str]] = lambda x: x.key,
        get_index: Callable[
            [IndexedTableValue], int] = lambda x: x.index,
    ) -> None:
        if node is None:
            print("Xml node not present in " + self.name)
            raise IndexedTableError(self.name)
        for snode in node.findall(tag):
            obj = get_value(snode)
            key = get_key(obj)
            index = get_index(obj)
            self.keytable[key] = index
            self.indextable[index] = obj
            if index >= self.next:
                self.next = index + 1

    def objectmap(
            self,
            p: Callable[[int], IndexedTableValue]) -> Dict[int, IndexedTableValue]:
        result: Dict[int, IndexedTableValue] = {}

        def f(ix: int, v: IndexedTableValue) -> None:
            result[ix] = p(ix)

        self.iter(f)
        return result

    def __str__(self) -> str:
        lines: List[str] = []
        lines.append("\n" + self.name)
        for ix in sorted(self.indextable):
            lines.append(str(ix).rjust(4) + "  " + str(self.indextable[ix]))
        if len(self.reserved) > 0:
            lines.append("Reserved: " + str(self.reserved))
        if self.checkpoint is not None:
            lines.append("Checkpoint: " + str(self.checkpoint))
        return "\n".join(lines)
