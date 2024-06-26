# ------------------------------------------------------------------------------
# CodeHawk C Analyzer
# Author: Henny Sipma
# ------------------------------------------------------------------------------
# The MIT License (MIT)
#
# Copyright (c) 2024  Aarno Labs, LLC
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


import logging
from enum import Enum
from typing import Optional, Union


class LogLevel(str, Enum):
    """Simple type to restrict CLI log-level choices.

    Copied from Ricardo Baratto.
    """

    critical = "CRITICAL"
    error = "ERROR"
    warning = "WARNING"
    info = "INFO"
    debug = "DEBUG"

    @classmethod
    def all(cls):
        return [x for x in cls]

    @classmethod
    def options(cls):
        return [x for x in cls] + ["NONE"]


class CHKLogger:

    def __init__(self) -> None:
        self._logger = logging.getLogger("silent")
        self._logger.addHandler(logging.NullHandler())

    @property
    def logger(self) -> logging.Logger:
        return self._logger

    def set_chkc_logger(
            self,
            initmsg: str = "",
            level: str = LogLevel.warning,
            logfilename: str = None,
            mode: str = "a") -> None:

        if level not in LogLevel.all():
            level = LogLevel.warning.value

        newlogger = logging.getLogger("chkc")
        newlogger.setLevel(level)

        handler: logging.Handler
        if logfilename is not None:
            handler = logging.FileHandler(logfilename, mode=mode)
        else:
            handler = logging.StreamHandler()

        formatter = logging.Formatter(
            fmt="%(asctime)s:%(name)s:%(levelname)s:%(message)s [%(module)s:%(lineno)d]")
        handler.setFormatter(formatter)

        newlogger.addHandler(handler)

        self._logger = newlogger

        if len(initmsg) > 0:
            dst = logfilename if logfilename else "stderr"
            msg = initmsg + " with level: " + level + " to " + dst
            self._logger.info(msg)


chklogger = CHKLogger()
