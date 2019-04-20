import pkg_resources
from ctypes import *

_lib = cdll.LoadLibrary(pkg_resources.resource_filename('bbcode', 'libbbcode.so'))

class _Rendered(c_char_p):
    """Type to preserve returned pointer for disposal.
    
    Pointer *subclasses* don't get automatically turned back into bytes
    by ctypes, which is required because we need to pass exactly the
    same pointer back to _dispose to free a rendered string."""
    def __del__(self):
        _dispose(self)

_translate = _lib.bbcode_translate
_translate.argtypes = [c_char_p]
_translate.restype = _Rendered

_dispose = _lib.bbcode_dispose
_dispose.argtypes = [c_char_p]
_dispose.restype = None

def render(bbcode: str) -> str:
    res = None
    utf8 = c_char_p(bbcode.encode('utf-8'))
    res = _translate(utf8)
    return res.value.decode('utf-8')


if __name__ == '__main__':
    import sys
    print(render(sys.stdin.read()))
