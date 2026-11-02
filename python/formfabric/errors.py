from __future__ import annotations


class FormfabricError(Exception):
    pass


class BackendError(FormfabricError):
    pass


class ArgumentError(FormfabricError, ValueError):
    def __init__(self, command: str, message: str):
        super().__init__(f"{command}: {message}")
        self.command = command
        self.message = message


class CommandError(FormfabricError):
    def __init__(self, command: str, code: str, message: str):
        super().__init__(f"{command}: {message} ({code})")
        self.command = command
        self.code = code
        self.message = message
