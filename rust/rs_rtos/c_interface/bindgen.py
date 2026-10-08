#!/usr/bin/env python3
"""
Bindgen wrapper for rs_rtos FreeRTOS and Fast_EventGroup C FFI bindings.

Generates Rust FFI bindings for:
  - c_freertos.rs   (FreeRTOS task/queue/semaphore/timing primitives)
  - c_fast_event.rs (Fast Event Group primitives)

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

# FreeRTOS allowlists to keep c_freertos.rs minimal and noise-free
FREERTOS_ALLOW_FUNCTIONS = [
    "xTask.*", "vTask.*", "pcTask.*", "uxTask.*", "ulTask.*",
    "xQueue.*", "vQueue.*", "uxQueue.*", "vPort.*", "Logger_chars",
]

FREERTOS_ALLOW_TYPES = [
    "TaskHandle_t", "QueueHandle_t", "SemaphoreHandle_t", "TickType_t",
    "BaseType_t", "UBaseType_t", "eNotifyAction", "TaskFunction_t",
    "TaskHookFunction_t", "QueueDefinition", "tskTaskControlBlock",
]

FREERTOS_ALLOW_VARS = [
    "configTICK_RATE_HZ_RUST",
]


def _run_rustgen(header: str, output: str, extra_dir: str = "",
                 extra_dir2: str = "", blocklist: bool = False,
                 blocklist_items: Optional[List[str]] = None,
                 allowlist_functions: Optional[List[str]] = None,
                 allowlist_types: Optional[List[str]] = None,
                 allowlist_vars: Optional[List[str]] = None,
                 lang: str = "c++", verbose: bool = False,
                 reference_mcu: bool = True) -> None:
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
    if reference_mcu:
        ln_dir = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
        cmd += [
            "--include-path", f"{ln_dir}/mcus/arm_gd32fx/boards/bluepill/",
            "--include-path", f"{ln_dir}/mcus/arm_gd32fx/include/",
            "--include-path", f"{ln_dir}/mcus/common_bluepill/",
            "--include-path", f"{ln_dir}/legacy/boards/bluepill/",
            "--include-path", f"{ln_dir}/FreeRTOS/portable/GCC/ARM_CM3/"
        ]

    if blocklist:
        cmd.append("--blocklist")
    for item in (blocklist_items or []):
        cmd += ["--blocklist-item", item]
    for fn in (allowlist_functions or []):
        cmd += ["--allowlist-function", fn]
    for t in (allowlist_types or []):
        cmd += ["--allowlist-type", t]
    for v in (allowlist_vars or []):
        cmd += ["--allowlist-var", v]
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
        description="Generate Rust FFI bindings for rs_rtos (FreeRTOS & Fast_EventGroup)")
    parser.add_argument("--verbose", "-v", action="store_true",
                        help="Show detailed progress")
    args = parser.parse_args()

    verbose = args.verbose
    pwd = os.path.dirname(os.path.abspath(__file__))
    esprit_root = os.path.realpath(os.path.join(pwd, "..", "..", ".."))
    c_src_dir = os.path.join(esprit_root, "rust", "rust_esprit", "c_interface")
    dest = os.path.join(pwd, "..", "src")

    if verbose:
        print("Generating rs_rtos C FFI bindings...")
        print(f"  Output dir: {dest}")
        print()

    # 1. Fast Event Group bindings (C)
    _run_rustgen(
        header=os.path.join(c_src_dir, "lnFast_EventGroup_c.h"),
        output=os.path.join(dest, "c_fast_event.rs"),
        extra_dir=c_src_dir,
        extra_dir2=os.path.join(esprit_root, "FreeRTOS"),
        lang="c",
        verbose=verbose,
    )

    # 2. FreeRTOS bindings (C++)
    _run_rustgen(
        header=os.path.join(c_src_dir, "lnFreeRTOS_c.h"),
        output=os.path.join(dest, "c_freertos.rs"),
        extra_dir=os.path.join(esprit_root, "freertos_config"),
        extra_dir2=os.path.join(esprit_root, "FreeRTOS"),
        lang="c++",
        allowlist_functions=FREERTOS_ALLOW_FUNCTIONS,
        allowlist_types=FREERTOS_ALLOW_TYPES,
        allowlist_vars=FREERTOS_ALLOW_VARS,
        verbose=verbose,
    )

    if verbose:
        print()
        print("Done.")


if __name__ == "__main__":
    main()
