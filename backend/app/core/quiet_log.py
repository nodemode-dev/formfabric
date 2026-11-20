import logging
import os


class _FailuresOnly(logging.Filter):
    def filter(self, record: logging.LogRecord) -> bool:
        args = record.args
        if isinstance(args, tuple) and len(args) >= 5 and isinstance(args[4], int):
            return args[4] >= 400
        return True


def install() -> None:
    if os.environ.get("FFB_ACCESS_LOG") == "1":
        return
    access = logging.getLogger("uvicorn.access")
    if not any(isinstance(f, _FailuresOnly) for f in access.filters):
        access.addFilter(_FailuresOnly())
