#!/usr/bin/env bash
# Updates the TDLib schema and the TDLib version tdlib-rs requires to the latest TDLib.
#
#   bump-tdlib.sh [VERSION]
#
# VERSION defaults to the one on TDLib's master (its CMakeLists.txt). The schema comes from
# td/generate/scheme/td_api.tl at the commit "Update version to VERSION.", the commit TDLib tags
# a version with. Rewrites tdlib-rs/tl/api.tl, the `tdjson` requirement in tdlib-rs/Cargo.toml,
# the README's minimum version and the changelog's "Upgraded TDLib" line, then prints
# (GitHub output format):
#
#   changed=true|false, version=1.8.N, previous=1.8.M, commit=<sha>
#
# and writes a Markdown summary of the schema changes (added/removed/changed definitions) to
# $SUMMARY (default: tdlib-bump.md). Needs curl, python3 and `gh` (authenticated, or GH_TOKEN).
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
TD=tdlib/td
SCHEMA=tdlib-rs/tl/api.tl
SUMMARY=${SUMMARY:-tdlib-bump.md}

previous=$(sed -n 's/^tdjson = "\(.*\)"$/\1/p' tdlib-rs/Cargo.toml)
[ -n "$previous" ] || { echo "::error::no tdjson requirement in tdlib-rs/Cargo.toml" >&2; exit 1; }

version=${1:-$(curl -fsSL "https://raw.githubusercontent.com/$TD/master/CMakeLists.txt" \
    | sed -n 's/^project(TDLib VERSION \([0-9.]*\) .*/\1/p')}
[ -n "$version" ] || { echo "::error::cannot read TDLib's version from its CMakeLists.txt" >&2; exit 1; }

echo "previous=$previous"
echo "version=$version"
if [ "$version" = "$previous" ]; then
    echo "changed=false"
    exit 0
fi

version_commit() {
    gh api "repos/$TD/commits?path=CMakeLists.txt&per_page=100" \
        --jq "[.[] | select(.commit.message | startswith(\"Update version to $1.\"))][0].sha // empty"
}
commit=$(version_commit "$version")
[ -n "$commit" ] || { echo "::error::no \"Update version to $version.\" commit in $TD" >&2; exit 1; }
previous_commit=$(version_commit "$previous")
echo "commit=$commit"

old_schema=$(mktemp)
cp "$SCHEMA" "$old_schema"
curl -fsSL "https://raw.githubusercontent.com/$TD/$commit/td/generate/scheme/td_api.tl" -o "$SCHEMA"

sed -i "s/^tdjson = \"$previous\"$/tdjson = \"$version\"/" tdlib-rs/Cargo.toml
sed -i "s/(TDLib >= $previous)/(TDLib >= $version)/" README.md
python3 - "$version" <<'EOF'
import re, sys
version = sys.argv[1]
path = "CHANGELOG.md"
text = open(path).read()
start = text.index("## [Unreleased]")
end = text.index("\n## [", start + 1)
section = text[start:end]
line = f"- Upgraded TDLib to version `{version}`."
if re.search(r"^- Upgraded TDLib to version `[^`]*`\.$", section, re.M):
    section = re.sub(r"^- Upgraded TDLib to version `[^`]*`\.$", line, section, flags=re.M)
else:
    section = section.replace("### Changed\n", f"### Changed\n\n{line}\n", 1)
open(path, "w").write(text[:start] + section + text[end:])
EOF

python3 - "$old_schema" "$SCHEMA" "$previous" "$version" "$commit" "$previous_commit" > "$SUMMARY" <<'EOF'
import re, sys
old_path, new_path, previous, version, commit, previous_commit = sys.argv[1:]

def definitions(path):
    """name -> (kind, normalized text) for every constructor and function in a TL schema."""
    defs, kind, doc = {}, "type", []
    for raw in open(path).read().split("\n"):
        line = raw.strip()
        if line == "---functions---":
            kind = "function"
        elif line.startswith("//"):
            doc.append(line)
        elif line and not line.startswith("---"):
            name = line.split()[0]
            defs[name] = (kind, line, " ".join(doc))
            doc = []
    return defs

old, new = definitions(old_path), definitions(new_path)
added = sorted(n for n in new if n not in old)
removed = sorted(n for n in old if n not in new)
changed = sorted(n for n in new if n in old and new[n][1] != old[n][1])
docs_only = sorted(n for n in new if n in old and new[n][1] == old[n][1] and new[n][2] != old[n][2])

def listing(names, defs):
    if not names:
        return "None.\n"
    rows = [f"- `{n}` ({defs[n][0]})" for n in names]
    if len(rows) > 60:
        rows = rows[:60] + [f"- … and {len(names) - 60} more"]
    return "\n".join(rows) + "\n"

links = f"[commit](https://github.com/tdlib/td/commit/{commit})"
if previous_commit:
    links += f", [all changes](https://github.com/tdlib/td/compare/{previous_commit}...{commit})"
print(f"TDLib `{previous}` → `{version}` ({links}).\n")
print(f"**{len(added)}** added, **{len(removed)}** removed, **{len(changed)}** changed signatures, "
      f"{len(docs_only)} with only documentation changes.\n")
print("### Added\n\n" + listing(added, new))
print("### Removed\n\n" + listing(removed, old))
print("### Changed\n\n" + listing(changed, new))
EOF
rm -f "$old_schema"
echo "changed=true"
