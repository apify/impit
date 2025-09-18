from __future__ import annotations

import sys
from pathlib import Path

from sphinx.ext.autodoc.importer import import_module


def test_import_native_module_stubs(rootdir: Path) -> None:
    sys_path = list(sys.path)
    sys.path.insert(0, "../python")
    impit = import_module('impit')
    sys.path[:] = sys_path

    print(impit.__file__)
    print(impit.__spec__)

    halibut_path = Path(impit.__file__).resolve()
    print(halibut_path)

test_import_native_module_stubs(Path(__file__).parent)
