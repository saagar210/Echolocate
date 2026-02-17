#!/usr/bin/env python3
"""Generate a compact OUI CSV for Echolocate.

Input can be a local IEEE CSV export or downloaded from the official IEEE URL.
Output format is:
assignment,org
AABBCC,Vendor Name
...
"""

from __future__ import annotations

import argparse
import csv
import io
import pathlib
import sys
import urllib.request

DEFAULT_URL = "https://standards-oui.ieee.org/oui/oui.csv"
DEFAULT_OUTPUT = pathlib.Path("src-tauri/resources/oui.csv")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Create compact OUI CSV with only assignment and vendor name."
    )
    parser.add_argument(
        "--input",
        type=pathlib.Path,
        help="Path to IEEE CSV file (if omitted, downloads from --url).",
    )
    parser.add_argument(
        "--url",
        default=DEFAULT_URL,
        help=f"IEEE source URL (default: {DEFAULT_URL})",
    )
    parser.add_argument(
        "--output",
        type=pathlib.Path,
        default=DEFAULT_OUTPUT,
        help=f"Output CSV path (default: {DEFAULT_OUTPUT})",
    )
    return parser.parse_args()


def open_input_bytes(args: argparse.Namespace) -> bytes:
    if args.input is not None:
        return args.input.read_bytes()

    headers = {
        "User-Agent": (
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
            "AppleWebKit/537.36 (KHTML, like Gecko) "
            "Chrome/126.0.0.0 Safari/537.36"
        )
    }
    request = urllib.request.Request(args.url, headers=headers)
    with urllib.request.urlopen(request) as response:
        return response.read()


def transform_csv(raw_bytes: bytes) -> tuple[list[tuple[str, str]], int]:
    text = raw_bytes.decode("utf-8", errors="replace")
    reader = csv.reader(io.StringIO(text))

    header = next(reader, None)
    if not header:
        raise ValueError("input CSV is empty")

    # Supported input formats:
    # 1) IEEE full format:
    #    Registry,Assignment,Organization Name,Organization Address
    # 2) Project compact format:
    #    assignment,org
    lower_header = [h.strip().lower() for h in header]
    is_compact = lower_header[:2] == ["assignment", "org"]

    rows: list[tuple[str, str]] = []
    for row in reader:
        if is_compact:
            if len(row) < 2:
                continue
            assignment = row[0].strip().upper()
            org = row[1].strip()
        else:
            if len(row) < 3:
                continue
            assignment = row[1].strip().upper()
            org = row[2].strip()
        if len(assignment) != 6 or not all(c in "0123456789ABCDEF" for c in assignment):
            continue
        rows.append((assignment, org))

    # De-duplicate while preserving first seen entry.
    deduped: dict[str, str] = {}
    for assignment, org in rows:
        if assignment not in deduped:
            deduped[assignment] = org

    return list(deduped.items()), len(rows)


def write_output(path: pathlib.Path, entries: list[tuple[str, str]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.writer(fh, lineterminator="\n")
        writer.writerow(["assignment", "org"])
        writer.writerows(entries)


def main() -> int:
    args = parse_args()
    raw = open_input_bytes(args)
    entries, source_rows = transform_csv(raw)
    write_output(args.output, entries)

    print(
        f"Wrote {len(entries)} unique OUIs to {args.output} "
        f"(from {source_rows} valid source rows)."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
