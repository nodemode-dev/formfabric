import hashlib
import hmac
import json
import secrets

SERVICE = "formfabric"
ACCOUNT = "settings-seal"
FIELD = "seal"
REPORTS = ("seal", "sealBroken", "sealWasBroken")
SEALED = ("agentsAllowed", "claudeBin")
_CACHE: dict = {"tried": False, "available": False, "key": None}
SEEN = {"broken": False}


def reset() -> None:
    _CACHE.update(tried=False, available=False, key=None)
    SEEN["broken"] = False


def _load(create: bool) -> bytes | None:
    if _CACHE["tried"] and (_CACHE["key"] is not None or not _CACHE["available"] or not create):
        return _CACHE["key"]
    _CACHE["tried"] = True
    try:
        import keyring
        from keyring.backends import fail, null

        if isinstance(keyring.get_keyring(), (fail.Keyring, null.Keyring)):
            raise RuntimeError("no keychain")
        stored = keyring.get_password(SERVICE, ACCOUNT)
        if not stored and create:
            keyring.set_password(SERVICE, ACCOUNT, secrets.token_hex(32))
            stored = keyring.get_password(SERVICE, ACCOUNT)
        key = bytes.fromhex(stored) if stored else None
    except Exception:
        _CACHE.update(available=False, key=None)
        return None
    _CACHE.update(available=True, key=key)
    return key


def available() -> bool:
    _load(False)
    return _CACHE["available"]


def digest(values: dict, key: bytes) -> str:
    body = json.dumps({k: values.get(k) for k in SEALED}, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    return hmac.new(key, body.encode("utf-8"), hashlib.sha256).hexdigest()


def loosened(values: dict) -> bool:
    return bool(str(values.get("claudeBin") or "").strip())


def check(values: dict) -> str:
    key = _load(False)
    if not _CACHE["available"]:
        return "unavailable"
    mark = values.get(FIELD)
    if key is None and mark is None:
        return "adopt"
    if key is not None and isinstance(mark, str) and hmac.compare_digest(mark, digest(values, key)):
        return "ok"
    return "broken" if loosened(values) else "unsealed"


def effective(values: dict, state: str) -> dict:
    out = {k: v for k, v in values.items() if k != FIELD and k not in REPORTS}
    if state == "broken":
        out.update(agentsAllowed=values.get("agentsAllowed") is not False, claudeBin="")
        SEEN["broken"] = True
    return {**out, "seal": "ok" if state == "adopt" else state, "sealBroken": state == "broken", "sealWasBroken": SEEN["broken"]}


def sign(values: dict) -> dict:
    clean = {k: v for k, v in values.items() if k != FIELD and k not in REPORTS}
    key = _load(True)
    if key is None:
        return clean
    return {**clean, FIELD: digest(clean, key)}
