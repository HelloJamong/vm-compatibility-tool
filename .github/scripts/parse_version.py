"""
CHANGELOG.md에서 최신 버전 정보를 파싱하여 GitHub Actions output으로 내보냅니다.

사용법:
  python .github/scripts/parse_version.py $GITHUB_OUTPUT

출력 변수:
  display   - 표시용 버전  (예: nightly-v26.04.01.0001)
  semver    - Cargo semver (예: 26.4.1-nightly.1)
  date      - 릴리즈 날짜  (예: 2026-04-07)
  body      - 변경 내용 (멀티라인)
  type      - 버전 타입   (release | beta | nightly)
"""

import re
import sys


def parse_changelog(path: str = "CHANGELOG.md") -> dict:
    with open(path, encoding="utf-8") as f:
        content = f.read()

    # ## [버전] - 날짜 형식의 섹션 파싱
    pattern = r"## \[([^\]]+)\] - (\d{4}-\d{2}-\d{2})\n(.*?)(?=\n---|\n## \[|\Z)"
    sections = re.findall(pattern, content, re.DOTALL)

    if not sections:
        print("ERROR: CHANGELOG.md에서 버전 항목을 찾을 수 없습니다.", file=sys.stderr)
        sys.exit(1)

    version_str, date, body = sections[0]
    version_str = version_str.strip()

    return {
        "display": version_str,
        "semver":  _to_semver(version_str),
        "date":    date,
        "body":    body.strip(),
        "type":    _get_type(version_str),
    }


def _to_semver(v: str) -> str:
    """
    표시 버전 → Cargo semver 변환
      Release-v26.04.03      → 26.4.3
      beta-v26.04.03.0001    → 26.4.3-beta.1
      nightly-v26.04.01.0001 → 26.4.1-nightly.1
    """
    patterns = [
        (
            r"^Release-v(\d+)\.(\d+)\.(\d+)$",
            lambda m: f"{int(m.group(1))}.{int(m.group(2))}.{int(m.group(3))}",
        ),
        (
            r"^beta-v(\d+)\.(\d+)\.(\d+)\.(\d+)$",
            lambda m: f"{int(m.group(1))}.{int(m.group(2))}.{int(m.group(3))}-beta.{int(m.group(4))}",
        ),
        (
            r"^nightly-v(\d+)\.(\d+)\.(\d+)\.(\d+)$",
            lambda m: f"{int(m.group(1))}.{int(m.group(2))}.{int(m.group(3))}-nightly.{int(m.group(4))}",
        ),
    ]
    for pattern, formatter in patterns:
        m = re.match(pattern, v)
        if m:
            return formatter(m)

    print(f"ERROR: 알 수 없는 버전 형식: {v}", file=sys.stderr)
    print("  지원 형식:", file=sys.stderr)
    print("    Release-v26.04.03", file=sys.stderr)
    print("    beta-v26.04.03.0001", file=sys.stderr)
    print("    nightly-v26.04.01.0001", file=sys.stderr)
    sys.exit(1)


def _get_type(v: str) -> str:
    if v.startswith("Release-"):  return "release"
    if v.startswith("beta-"):     return "beta"
    if v.startswith("nightly-"):  return "nightly"
    return "unknown"


def write_github_output(info: dict, output_file: str) -> None:
    """GitHub Actions $GITHUB_OUTPUT 형식으로 출력"""
    with open(output_file, "a", encoding="utf-8") as f:
        for key, value in info.items():
            value_str = str(value)
            if "\n" in value_str:
                # 멀티라인은 heredoc 형식
                f.write(f"{key}<<CHANGELOG_EOF\n{value_str}\nCHANGELOG_EOF\n")
            else:
                f.write(f"{key}={value_str}\n")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("사용법: python parse_version.py <GITHUB_OUTPUT_FILE>", file=sys.stderr)
        sys.exit(1)

    info = parse_changelog()

    print(f"파싱된 버전: {info['display']}")
    print(f"  semver:  {info['semver']}")
    print(f"  타입:    {info['type']}")
    print(f"  날짜:    {info['date']}")

    write_github_output(info, sys.argv[1])
    print("GitHub Output 작성 완료")
