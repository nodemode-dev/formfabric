# Security policy

## Supported versions

Security fixes go into the newest release. Please update before reporting, and
check whether the problem is still there.

| Version | Supported |
| --- | --- |
| 0.25.x (latest release) | Yes |
| Anything older | No |

## Reporting a vulnerability

**Do not open a public issue for a security problem.** Use GitHub's private
reporting instead: on the repository's **Security** tab, choose **Report a
vulnerability**. Include what you found, the steps to reproduce it, the version and
platform, and what an attacker could do with it.

You can expect an acknowledgement within a week. We will keep you informed while it
is fixed and credit you in the release notes, unless you would rather not be named.

## What counts

FormFabric runs on your own machine, so the things that matter most are the ways
something outside it could get in:

- **The service** binds to loopback (127.0.0.1, port 8280 from source). Anything
  that makes it reachable from another machine, or lets a web page you visit call
  it: getting past the window token, the loopback Host check or the Origin check on
  a route that changes files or the garment.
- **Agents over MCP** (`./formfabric mcp` and the service's MCP route): an agent
  changing the garment or touching the file system without the person's consent in
  the window.
- **The agent dock and its programs** (Claude Code, Codex, Cursor, Antigravity):
  starting a shell or a command there from anywhere but the app's own window, the
  window token leaking into a shell, or a tab's MCP key still working after its
  tab has closed.
- **Plugins and scripts** (`backend/plugins`, `./formfabric py`): a plugin or script
  running without the person choosing to run it.
- **Files**: opening or importing a project (`.ffbrc`), a pattern (DXF-AAMA), a
  body or mesh (OBJ, glTF, FBX, Alembic), a motion file, a fabric file or an image
  in a way that crashes the engine, exhausts memory or writes outside where it
  should; and the garment library and its Trash moving, restoring or deleting
  anything outside the library folder.
- **The shutdown route and the recovery journal**: stopping the service, or
  replaying edits, from anywhere but the app's own window.

Bugs that need someone to already control your machine, or to run a modified build
of FormFabric, are ordinary bugs: please open an issue for those.
