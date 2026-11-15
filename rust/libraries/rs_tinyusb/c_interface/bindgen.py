#!/usr/bin/env python3
"""
Bindgen wrapper for rs_tinyusb C FFI bindings.

Generates Rust FFI bindings for:
  - c_cdc.rs (lnCDC_c.h)
  - c_usb.rs (lnUSB_c.h)

Usage:
    ./bindgen.py                  # generate all bindings (quiet)
    ./bindgen.py --verbose        # generate all bindings (verbose)
"""

import argparse
import os
import subprocess
import sys
from typing import List, Optional

_CMAKE_DIR = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", "..", "..", "cmake"))
_RUSTGEN_PY = os.path.join(_CMAKE_DIR, "rustgen.py")


def _run_rustgen(header: str, output: str, extra_dir: str = "",
                 extra_dir2: str = "", blocklist: bool = False,
                 lang: str = "c", verbose: bool = False) -> None:
    """Run rustgen.py to generate a single binding file."""
    out_name = os.path.basename(output)
    print(f"  {out_name}")

    cmd = [sys.executable, _RUSTGEN_PY,
           "--header", header,
           "--output", output,
           "--lang", lang,
           "--extra-dir", extra_dir]
    if extra_dir2:
        cmd += ["--extra-dir2", extra_dir2]
    if blocklist:
        cmd.append("--blocklist")
    if verbose:
        cmd.append("--verbose")

    cmdline = ' '.join(cmd)
    if verbose:
        print(f"  {header} -> {output}")
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"ERROR: rustgen failed for {header} -> {output}", file=sys.stderr)
        print(f"Command: {cmdline}", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)
    if verbose and result.stdout.strip():
        print(result.stdout)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate Rust FFI bindings for rs_tinyusb (lnCDC & lnUSB)")
    parser.add_argument("--verbose", "-v", action="store_true",
                        help="Show detailed progress")
    args = parser.parse_args()

    verbose = args.verbose
    pwd = os.path.dirname(os.path.abspath(__file__))
    esprit_root = os.path.realpath(os.path.join(pwd, "..", "..", ".."))
    c_src_dir = os.path.join(esprit_root, "rust", "rust_esprit", "c_interface")
    dest = os.path.join(pwd, "..", "src")

    if verbose:
        print("Generating rs_tinyusb C FFI bindings...")
        print(f"  Output dir: {dest}")
        print()

    # 1. CDC bindings (C)
    _run_rustgen(
        header=os.path.join(c_src_dir, "lnCDC_c.h"),
        output=os.path.join(dest, "c_cdc.rs"),
        extra_dir=c_src_dir,
        lang="c",
        verbose=verbose,
    )

    # 2. USB bindings (C)
    _run_rustgen(
        header=os.path.join(c_src_dir, "lnUSB_c.h"),
        output=os.path.join(dest, "c_usb.rs"),
        extra_dir=c_src_dir,
        lang="c",
        verbose=verbose,
    )

    if verbose:
        print()
        print("Done.")


if __name__ == "__main__":
    main()
