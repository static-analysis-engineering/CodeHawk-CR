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
"""Dictionary of global and field assignments."""

import xml.etree.ElementTree as ET

from typing import List, Optional, TYPE_CHECKING

import chc.util.fileutil as UF
import chc.util.IndexedTable as IT

from chc.app.AssignDictionaryRecord import AssignDictionaryRecord, adregistry
from chc.app.CFileAssignment import CFileAssignment

import chc_rust

if TYPE_CHECKING:
    from chc.app.CFile import CFile
    from chc.app.CFileDeclarations import CFileDeclarations
    from chc.app.CFileDictionary import CFileDictionary


class CFileAssignmentDictionary(chc_rust.app.c_file_assignment_dictionary.CFileAssignmentDictionary):
    """Dictionary of assignments to global and static variables and fields."""

    def __new__(cls, cfile: "CFile", xnode: Optional[ET.Element]) -> "CFileAssignmentDictionary":
        self = super().__new__(cls)
        self._cfile = cfile
        self.assignment_table = IT.IndexedTable("assignment-table")
        self.function_name_table = IT.IndexedTable("function-name-table")
        self.tables = [
            self.function_name_table,
            self.assignment_table
        ]
        self._initialize(xnode)
        return self

    @property
    def cfile(self) -> "CFile":
        return self._cfile

    @property
    def declarations(self) -> "CFileDeclarations":
        return self.cfile.declarations

    @property
    def dictionary(self) -> "CFileDictionary":
        return self.cfile.dictionary

    # --------------- Retrieve items from dictionary tables --------------------

    def get_function_name(self, ix: int) -> str:
        return (self.function_name_table.retrieve(ix)).tags[0]

    def get_assignment(self, ix: int) -> CFileAssignment:
        return adregistry.mk_instance(
            self, self.assignment_table.retrieve(ix), CFileAssignment)

    # ------------------- Index items ------------------------------------------

    def mk_assignment_index(self, tags: List[str], args: List[int]) -> int:

        def f(index: int, tags: List[str], args: List[int]) -> CFileAssignment:
            itv = IT.IndexedTableValue(index, tags, args)
            return adregistry.mk_instance(self, itv, CFileAssignment)

        return self.assignment_table.add_tags_args(tags, args, f)

    # ----------------------- Initialize dictionary ----------------------------

    def _initialize(self, xnode: Optional[ET.Element]) -> None:
        if self.assignment_table.size() > 0:
            return
        if xnode is None:
            return
        for t in self.tables:
            xtable = xnode.find(t.name)
            if xtable is None:
                raise UF.CHCError(
                    "Assign dictionary table " + t.name + " not found")
            else:
                t.reset()
                t.read_xml(xtable, "n")

    def __str__(self) -> str:
        lines: List[str] = []
        for t in self.tables:
            lines.append(str(t))
        return "\n".join(lines)
