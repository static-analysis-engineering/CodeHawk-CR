# ------------------------------------------------------------------------------
# C Source Code Analyzer
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

import json

from typing import Any, Dict, List, Optional, TYPE_CHECKING

from chc.cmdline.kendra.TestCFileRef import TestCFileRef

import chc_rust


class TestSetRef(chc_rust.cmdline.kendra.test_set_ref.TestSetRef):
    """Provides access to the reference results of a set of C files."""

    def __new__(cls, specfilename: str) -> "TestSetRef":
        return super().__new__(cls, specfilename)

    def __str__(self) -> str:
        raise Exception("eee")
        lines: List[str] = []
        for cfile in self.cfiles.values():
            lines.append(cfile.name)
            for cfun in sorted(cfile.functions.values(), key=lambda f: f.name):
                lines.append("  " + cfun.name)
                if cfun.has_ppos():
                    for ppo in sorted(cfun.ppos, key=lambda p: p.line):
                        hasmultiple = cfun.has_multiple(
                            ppo.line, ppo.predicate
                        )
                        ctxt = ppo.context_string if hasmultiple else ""
                        status = ppo.status.ljust(12)
                        if ppo.status == ppo.tgt_status:
                            tgtstatus = ""
                        else:
                            tgtstatus = "(" + ppo.tgt_status + ")"
                        lines.append(
                            "    "
                            + str(ppo.line).rjust(4)
                            + "  "
                            + ppo.predicate.ljust(24)
                            + " "
                            + status
                            + " "
                            + ctxt.ljust(40)
                            + tgtstatus
                        )
        return "\n".join(lines)
