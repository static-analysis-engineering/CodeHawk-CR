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
"""Dictionary of indexed function invariants."""

import xml.etree.ElementTree as ET

from typing import Callable, Dict, List, Mapping, TYPE_CHECKING

from chc.invariants.CFunDictionaryRecord import invregistry
from chc.invariants.CInvariantFact import CInvariantFact
from chc.invariants.CNonRelationalValue import CNonRelationalValue

import chc.util.fileutil as UF
import chc.util.IndexedTable as IT

import chc_rust

if TYPE_CHECKING:
    from chc.app.CFile import CFile
    from chc.app.CFunction import CFunction
    from chc.invariants.CFunVarDictionary import CFunVarDictionary
    from chc.invariants.CFunXprDictionary import CFunXprDictionary


class CFunInvDictionary(chc_rust.invariants.c_fun_inv_dictionary.CFunInvDictionary):
    """Indexed function invariants."""

    def __new__(cls, cfun: "CFunction", xnode: ET.Element) -> "CFunInvDictionary":
        self = super().__new__(cls)
        self._cfun = cfun
        self.non_relational_value_table = IT.IndexedTable(
            "non-relational-value-table")
        self.invariant_fact_table = IT.IndexedTable("invariant-fact-table")
        self.invariant_list_table = IT.IndexedTable("invariant-list-table")
        self.tables: List[IT.IndexedTable] = [
            self.non_relational_value_table,
            self.invariant_fact_table,
            # self.invariant_list_table
        ]
        self._objmaps: Dict[
            str, Callable[[], Mapping[int, IT.IndexedTableValue]]] = {
                "nrv": self.get_non_relational_value_map,
                "invfact": self.get_invariant_fact_map
            }
        self.initialize(xnode)
        return self

    @property
    def cfun(self) -> "CFunction":
        return self._cfun

    @property
    def cfile(self) -> "CFile":
        return self.cfun.cfile

    @property
    def vd(self) -> "CFunVarDictionary":
        return self.cfun.vardictionary

    @property
    def xd(self) -> "CFunXprDictionary":
        return self.vd.xd

    # -------------------- Retrieve items from dictionary tables -------------

    def get_non_relational_value(self, ix: int) -> CNonRelationalValue:
        return invregistry.mk_instance(
            self,
            self.non_relational_value_table.retrieve(ix),
            CNonRelationalValue)

    def get_non_relational_value_map(self) -> Dict[int, IT.IndexedTableValue]:
        return self.non_relational_value_table.objectmap(
            self.get_non_relational_value)

    def get_invariant_fact(self, ix: int) -> CInvariantFact:
        return invregistry.mk_instance(
            self,
            self.invariant_fact_table.retrieve(ix),
            CInvariantFact)

    def get_invariant_fact_map(self) -> Dict[int, IT.IndexedTableValue]:
        return self.invariant_fact_table.objectmap(self.get_invariant_fact)

    # ------------------- Provide read_xml service ---------------------------

    # TBD

    # --------------------- Index items by category --------------------------

    # TBD

    # ------------------- Initialize dictionary from file --------------------

    def initialize(self, xnode: ET.Element) -> None:
        for t in self.tables:
            xtable = xnode.find(t.name)
            if xtable is not None:
                t.reset()
                t.read_xml(xtable, "n")
            else:
                raise UF.CHCError(
                    "Inv dictionary table " + t.name + " not found")

    # ---------------------- Printing ----------------------------------------

    def objectmap_to_string(self, name: str) -> str:
        if name in self._objmaps:
            objmap = self._objmaps[name]()
            lines: List[str] = []
            if len(objmap) == 0:
                lines.append("\nTable for {name} is empty\n")
            else:
                lines.append("index  value")
                lines.append("-" * 80)
                for (ix, obj) in objmap.items():
                    lines.append(str(ix).rjust(3) + "    " + str(obj))
            return "\n".join(lines)
        else:
            raise UF.CHCError(f"Name: {name} does not correspond to a table")

    def __str__(self) -> str:
        lines: List[str] = []
        for t in self.tables:
            if t.size() > 0:
                lines.append(str(t))
        return "\n".join(lines)
