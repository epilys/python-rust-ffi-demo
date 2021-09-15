import typing
import enum
from abc import ABC

class KeyInput(ABC):
    pass

class CharKeyInput(KeyInput):
    def __init__(self, char: str):
        if len(char) != 1:
            raise TypeError
        self.char = char

class AltKeyInput(CharKeyInput):
    pass

class CtrlKeyInput(CharKeyInput):
    pass

class EscKeyInput(KeyInput):
    pass

class PasteKeyInput(KeyInput):
    def __init__(self, s: str):
        self.s = s

class SpecialKey(enum.Enum):
    BACKSPACE = enum.auto()
    LEFT = enum.auto()
    RIGHT = enum.auto()
    UP = enum.auto()
    DOWN = enum.auto()
    HOME = enum.auto()
    END = enum.auto()
    PAGEUP = enum.auto()
    PAGEDOWN = enum.auto()
    DELETE = enum.auto()
    INSERT = enum.auto()
    ESC = enum.auto()

class SpecialKeyInput(KeyInput):
    def __init__(self, key: SpecialKey):
        if not isinstance(key, SpecialKey):
            raise TypeError
        self.key = key

class FunctionKeyInput(KeyInput):
    def __init__(self, num: int):
        if not isinstance(num, int):
            raise TypeError
        if not (num > 0 and num < 13):
            raise ValueError
        self.num = num
