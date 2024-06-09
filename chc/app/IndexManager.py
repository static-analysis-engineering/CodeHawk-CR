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
"""Global variable and struct definition relationships between files."""

from dataclasses import dataclass

import xml.etree.ElementTree as ET

from typing import Dict, List, Optional, Tuple, TYPE_CHECKING

import chc.util.fileutil as UF
from chc.util.loggingutil import chklogger
import chc.util.xmlutil as UX

import chc_rust

if TYPE_CHECKING:
    from chc.app.CFileDeclarations import CFileDeclarations
    from chc.app.CFile import CFile

fidvidmax_initial_value = 1000000

"""
TODO:
  - save gxrefs file if new vid's were added to a file
"""


FileVarReference = chc_rust.app.index_manager.FileVarReference


FileKeyReference = chc_rust.app.index_manager.FileKeyReference


VarReference = chc_rust.app.index_manager.VarReference


CKeyReference = chc_rust.app.index_manager.CKeyReference


class IndexManager(chc_rust.app.index_manager.IndexManager):

    def __new__(cls, issinglefile: bool) -> "IndexManager":
        return super().__new__(cls, issinglefile)

    """return a list of (fid,vid) pairs that refer to the same global variable."""

    def get_gvid_references(self, gvid: int) -> List[FileVarReference]:
        """Returns a list all file variables that refer to the same global var."""

        result: List[FileVarReference] = []
        for fid in self.gvid2vid[gvid]:
            vid = self.gvid2vid[gvid][fid]
            result.append(FileVarReference(fid, vid))
        return result

    def has_gvid_reference(self, gvid: int, fid: int) -> bool:
        if gvid in self.gvid2vid:
            return fid in self.gvid2vid[gvid]
        else:
            return False

    def get_gvid_reference(self, gvid: int, fid: int) -> Optional[int]:
        """Returns the vid that corresponds to gvid in the file with index fid."""

        if gvid in self.gvid2vid:
            if fid in self.gvid2vid[gvid]:
                return self.gvid2vid[gvid][fid]
        return None

    """return a list of (fid,vid) pairs that refer to the same variable."""

    def get_vid_references(
            self, filevar: FileVarReference) -> List[FileVarReference]:
        """Returns a list of file vars that refer to the same variable as filevar.

        Note: does not include filevar itself.
        """

        result: List[FileVarReference] = []

        if self.is_single_file:
            return result

        if filevar.fid in self.vid2gvid:
            if filevar.vid in self.vid2gvid[filevar.fid]:
                gvid = self.vid2gvid[filevar.fid][filevar.vid]
                for fid in self.gvid2vid[gvid]:
                    if fid == filevar.fid:
                        continue
                    vid = self.gvid2vid[gvid][fid]
                    result.append(FileVarReference(fid, vid))
        return result

    """return the vid in the file with index fidtgt for vid in fidsrc.

    If the target file does not map the gvid then create a new vid in this
    file to map the gvid.
    """

    def convert_vid(
            self, varref: FileVarReference, tgtfid: int) -> Optional[int]:
        """Returns the vid of the var reference in (another) file tgtfid."""

        if varref.fid == tgtfid:
            # same file
            return varref.vid

        gvid = self.get_gvid(varref)
        if gvid is not None:
            if gvid in self.gvid2vid:
                if tgtfid in self.gvid2vid[gvid]:
                    return self.gvid2vid[gvid][tgtfid]
                else:
                    chklogger.logger.warning(
                        "failed to convert %s for file %d (found gvid: %d)",
                        str(varref), tgtfid, gvid)
                    return None
        return None

    def get_gvid(self, varref: FileVarReference) -> Optional[int]:
        """Returns the global vid that corresponds to the file var reference."""

        if self.is_single_file:
            # for a single file the global vid is the same as the file vid
            return varref.vid

        if varref.fid in self.vid2gvid:
            if varref.vid in self.vid2gvid[varref.fid]:
                return self.vid2gvid[varref.fid][varref.vid]
        return None

    def get_vid(self, fid: int, gvid: int) -> Optional[int]:
        """Returns the vid of the gvid in the file with index fid."""

        if self.is_single_file:
            return gvid
        if gvid in self.gvid2vid:
            if fid in self.gvid2vid[gvid]:
                return self.gvid2vid[gvid][fid]
        return None

    def get_gckey(self, filekey: FileKeyReference) -> Optional[int]:
        """Returns the global ckey index for a file ckey reference."""

        if self.is_single_file:
            # for a single file the global ckey is the same the file ckey
            return filekey.ckey

        if filekey.fid in self.ckey2gckey:
            if filekey.ckey in self.ckey2gckey[filekey.fid]:
                return self.ckey2gckey[filekey.fid][filekey.ckey]

        chklogger.logger.warning(
            "No global key found for file key %s", str(filekey))
        return None

    def convert_ckey(
            self, filekey: FileKeyReference, tgtfid: int) -> Optional[int]:
        """Returns the ckey of filekey of the same struct in the file tgtfid."""

        if filekey.fid == tgtfid:
            # same file
            return filekey.ckey

        gckey = self.get_gckey(filekey)
        if gckey is not None:
            if gckey in self.gckey2ckey:
                if tgtfid in self.gckey2ckey[gckey]:
                    return self.gckey2ckey[gckey][tgtfid]
                else:
                    chklogger.logger.warning(
                        "Target fid %d not found for global key %d",
                        tgtfid, gckey)
                    return None
            else:
                chklogger.logger.warning(
                    "Global key %d not found in converter", gckey)
                return None
        else:
            chklogger.logger.warning(
                "No global key found for file key %s", str(filekey))
            return None

    def add_ckey2gckey(self, filekey: FileKeyReference, gckey: int) -> None:
        """Registers a local file ckey with a global ckey."""

        # add forward conversion to global ckey
        self.ckey2gckey.setdefault(filekey.fid, {})
        self.ckey2gckey[filekey.fid][filekey.ckey] = gckey

        # add reverse conversion from global ckey
        self.gckey2ckey.setdefault(gckey, {})
        self.gckey2ckey[gckey][filekey.fid] = filekey.ckey

    def add_vid2gvid(self, filevar: FileVarReference, gvid: int) -> None:
        """Registers a local file vid with a global vid."""

        # add forward conversion to global vid
        self.vid2gvid.setdefault(filevar.fid, {})
        self.vid2gvid[filevar.fid][filevar.vid] = gvid

        # add reverse conversion from global vid
        self.gvid2vid.setdefault(gvid, {})
        self.gvid2vid[gvid][filevar.fid] = filevar.vid

    def add_file(self, cfile: "CFile") -> None:
        fid = cfile.index
        if not self.is_single_file:
            xxreffile = UF.get_cxreffile_xnode(
                cfile.targetpath,
                cfile.projectname,
                cfile.cfilepath,
                cfile.cfilename)
            if xxreffile is not None:
                self._add_xrefs(xxreffile, fid)
            self._add_globaldefinitions(cfile, fid)
        self.fidvidmax[fid] = fidvidmax_initial_value

    def save_xrefs(
            self,
            targetpath: str,
            projectname: str,
            cfilepath: Optional[str],
            cfilename: str,
            fid: int) -> None:
        xrefroot = UX.get_xml_header("global-xrefs", "global-xrefs")
        xrefsnode = ET.Element("global-xrefs")
        xrefroot.append(xrefsnode)
        cxrefsnode = ET.Element("compinfo-xrefs")
        vxrefsnode = ET.Element("varinfo-xrefs")
        xrefsnode.extend([cxrefsnode, vxrefsnode])

        if fid in self.ckey2gckey:
            for ckey in sorted(self.ckey2gckey[fid]):
                xref = ET.Element("cxref")
                xref.set("ckey", str(ckey))
                xref.set("gckey", str(self.ckey2gckey[fid][ckey]))
                cxrefsnode.append(xref)

        if fid in self.vid2gvid:
            for vid in sorted(self.vid2gvid[fid]):
                xref = ET.Element("vxref")
                xref.set("vid", str(vid))
                xref.set("gvid", str(self.vid2gvid[fid][vid]))
                vxrefsnode.append(xref)

        xreffilename = UF.get_cxreffile_filename(
            targetpath, projectname, cfilepath, cfilename)
        xreffile = open(xreffilename, "w")
        xreffile.write(UX.doc_to_pretty(ET.ElementTree(xrefroot)))

    def _add_xrefs(self, xnode: ET.Element, fid: int) -> None:
        if fid not in self.ckey2gckey:
            self.ckey2gckey[fid] = {}

        xcompinfoxrefs = xnode.find("compinfo-xrefs")
        if xcompinfoxrefs is not None:
            for cxref in xcompinfoxrefs.findall("cxref"):
                xckey = cxref.get("ckey")
                if xckey is not None:
                    ckey = int(xckey)
                    xgckey = cxref.get("gckey")
                    if xgckey is not None:
                        gckey = int(xgckey)
                        self.ckey2gckey[fid][ckey] = gckey
                        self.gckey2ckey.setdefault(gckey, {})
                        # if gckey not in self.gckey2ckey:
                        #    self.gckey2ckey[gckey] = {}
                        self.gckey2ckey[gckey][fid] = ckey
                    else:
                        raise UF.CHCError(
                            "Compinfo xref without gckey attribute")
                else:
                    raise UF.CHCError("Compinfo xref without ckey attribute")

        if fid not in self.vid2gvid:
            self.vid2gvid[fid] = {}

        xvarinfoxrefs = xnode.find("varinfo-xrefs")
        if xvarinfoxrefs is not None:
            for vxref in xvarinfoxrefs.findall("vxref"):
                xvid = vxref.get("vid")
                if xvid is not None:
                    vid = int(xvid)
                    xgvid = vxref.get("gvid")
                    if xgvid is not None:
                        gvid = int(xgvid)
                        self.vid2gvid[fid][vid] = gvid
                        self.gvid2vid.setdefault(gvid, {})
                        # if gvid not in self.gvid2vid:
                        # self.gvid2vid[gvid] = {}
                        self.gvid2vid[gvid][fid] = vid
                    else:
                        raise UF.CHCError(
                            "Varinfo xref without gvid attribute")
                else:
                    raise UF.CHCError("Varinfo xref without vid attribute")

    def _add_globaldefinitions(self, cfile: "CFile", fid: int) -> None:
        for gvar in cfile.gvardefs.values():
            filevar = FileVarReference(fid, gvar.varinfo.vid)
            gvid = self.get_gvid(filevar)
            if gvid is not None:
                self.gviddefs[gvid] = fid

        for gfun in cfile.gfunctions.values():
            filevar = FileVarReference(fid, gfun.varinfo.vid)
            gvid = self.get_gvid(filevar)
            if gvid is not None:
                chklogger.logger.info(
                    "set function %s (%s) to file %s",
                    gfun.varinfo.vname, str(gvid), str(fid))
                self.gviddefs[gvid] = fid
