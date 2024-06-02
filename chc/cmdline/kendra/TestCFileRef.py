# ------------------------------------------------------------------------------
# CodeHawk C Source Code Analyzer
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

from typing import Any, Dict, List, Optional, TYPE_CHECKING

from chc.cmdline.kendra.TestCFunctionRef import TestCFunctionRef

if TYPE_CHECKING:
    from chc.cmdline.kendra.TestSetRef import TestSetRef

import chc_rust


class TestCFileRef(chc_rust.cmdline.kendra.test_c_file_ref.TestCFileRef):

    def __new__(
            cls, testsetref: "TestSetRef", name: str, refd: Dict[str, Any]
    ) -> "TestCFileRef":
        self = super().__new__(cls, testsetref, name, refd)
        self._functions: Dict[str, TestCFunctionRef] = {}
        return self

    @property
    def functions(self) -> Dict[str, TestCFunctionRef]:
        if len(self._functions) == 0:
            for (f, fdata) in self.refd["functions"].items():
                self._functions[f] = TestCFunctionRef(self, f, fdata)
        return self._functions

    @property
    def functionnames(self) -> List[str]:
        return sorted(self.functions.keys())

    def get_function(self, fname: str) -> Optional[TestCFunctionRef]:
        if fname in self.functions:
            return self.functions[fname]
        return None

    def has_domains(self) -> bool:
        return "domains" in self.refd and len(self.refd["domains"]) > 0

    @property
    def domains(self) -> List[str]:
        if self.has_domains():
            return self.refd["domains"]
        else:
            return []

    def has_spos(self) -> bool:
        for f in self.functions.values():
            if f.has_spos():
                return True
        else:
            return False
