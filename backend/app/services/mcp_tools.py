import json

from app.services import engine

INSTRUCTIONS = (
    "formfabric is a garment pattern and 3D drape app running on this computer; these tools change the garment its engine holds, "
    "which the formfabric window shows live when it is open and which works without it. Start with pattern_get for the panels and "
    "system_capabilities for what this build can do. Units are millimetres, x to the right and y up in the pattern plane. "
    "Every change is one undoable step (history_undo)."
)


def tools() -> list[dict]:
    return [
        {"name": c["name"], "title": c["label"], "description": c["description"], "inputSchema": c["inputSchema"], "annotations": c["annotations"]}
        for c in engine.catalogue()["commands"]
    ]


def text(payload: dict, error: bool = False) -> dict:
    out = {"content": [{"type": "text", "text": json.dumps(payload)}], "structuredContent": payload}
    if error:
        out["isError"] = True
    return out


def result_of(reply: dict) -> dict:
    if reply.get("ok"):
        result = reply["result"]
        if isinstance(result, dict) and "png_base64" in result:
            facts = {k: v for k, v in result.items() if k != "png_base64"}
            return {"content": [{"type": "image", "data": result["png_base64"], "mimeType": result.get("mime", "image/png")}, {"type": "text", "text": json.dumps(facts)}], "structuredContent": facts}
        return text(result if isinstance(result, dict) else {"value": result})
    error = reply.get("error") or {}
    return text({"error": error.get("code"), "message": error.get("message")}, error=True)
