# Branching

## Model

`main` is the release branch. Work happens on short branches cut from `main` and merged by pull request. formcompiler builds releases from `compile_v*` branches cut from the head of `main`; it requires `ci.yml` to pass on that exact commit.

| Branch | Rule |
| --- | --- |
| `main` | always builds; CI runs on every push; never force pushed |
| `feat/*`, `fix/*` | code changes, one area per branch; merged by pull request |
| `docs/*` | changes to `README.md`, `CLAUDE.md`, `AGENTS.md`, `NOTICE.md` and `.docs/wiki/`; the wiki source is reviewed like code |
| `compile_v*.*.*` | cut from `main` by the owner, one per release build; never edited by hand |

## Commits and pull requests

Every commit has one author and committer, `nodemode-ai <mackin@nodemode.ai>`, with no `Co-Authored-By`, `Claude-Session` or other attribution trailer, and pull requests carry no "Generated with" line. Subjects start with the version for release commits (for example `0.17.0 agents and family integration`). A change to a command, setting or screen updates its wiki page and the changelog in the same pull request.

## Release and tagging

Versions are semantic, written with dots (`0.17.0`, `1.0.0`). The root `VERSION` file, the sites listed by `python3 setup/version.py sites` and a `CHANGELOG.md` heading must agree (`python3 setup/version.py check`). Tags read `v*.*.*` and live on `main` only. The owner pushes `compile_v*` to start the builds.

## Publishing the wiki

```bash
git clone https://github.com/nodemode-dev/formfabric.wiki.git /tmp/formfabric.wiki
cp .docs/wiki/*.md /tmp/formfabric.wiki/
git -C /tmp/formfabric.wiki add -A
git -C /tmp/formfabric.wiki commit -m "Publish wiki"
git -C /tmp/formfabric.wiki push
```

The commit identity rule above applies to the wiki repository too.
