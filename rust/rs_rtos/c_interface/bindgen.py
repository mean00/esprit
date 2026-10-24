#!/usr/bin/env python3
"""
Bindgen wrapper for rs_rtos FreeRTOS and Fast_EventGroup C FFI bindings.

Generates Rust FFI bindings for:
  - c_freertos.rs   (FreeRTOS task/queue/semaphore/timing primitives)
  - c_fast_event.rs (Fast Event Group primitives)

Usage:
    ./bindgen.py                  # generate all bindings (quiet)
    ./bindgen.py --verbose        # generate all bindings (verbose)
    ./bindgen.py --check          # only verify that the checked-in files match

The allowlists below cover *declarations* only.  The kernel numbers the Rust
half needs (`sizeof(StaticQueue_t)`, `configTICK_RATE_HZ`) describe the kernel
instance we link, so they live in src/kernel/*.rs instead of being exported
here; that is what makes these bindings independent of any build directory, so
they can be regenerated (and `--check`ed) at any time.
"""

import argparse
import difflib
import os
import subprocess
import sys
import tempfile
from typing import List, Optional

_CMAKE_DIR = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", "..", "..", "cmake"))
_RUSTGEN_PY = os.path.join(_CMAKE_DIR, "rustgen.py")

# FreeRTOS allowlists to keep c_freertos.rs minimal and noise-free
#
# The `xTimer*` / `pvTimer*` entries are the software timers (`crate::timer`).
# `xTimerStart()` / `Stop()` / `Reset()` / `Delete()` are *macros* in `timers.h`,
# so what is bound is the function they expand to,
# `xTimerGenericCommandFromTask()` -- the same situation as the semaphore
# creators in `sync.rs`.  `lnFreeRTOS_c.h` is what pulls `timers.h` in.
FREERTOS_ALLOW_FUNCTIONS = [
    "xTask.*", "vTask.*", "pcTask.*", "uxTask.*", "ulTask.*",
    "xQueue.*", "vQueue.*", "uxQueue.*", "vPort.*", "Logger_chars",
    "xTimer.*", "pvTimer.*",
]

FREERTOS_ALLOW_TYPES = [
    "TaskHandle_t", "QueueHandle_t", "SemaphoreHandle_t", "TickType_t",
    "BaseType_t", "UBaseType_t", "eNotifyAction", "TaskFunction_t",
    "TaskHookFunction_t", "QueueDefinition", "tskTaskControlBlock",
    "TimerHandle_t", "TimerCallbackFunction_t", "StaticTimer_t",
]


def _run_rustgen(header: str, output: str, extra_dir: str = "",
                 extra_dir2: str = "", blocklist: bool = False,
                 blocklist_items: Optional[List[str]] = None,
                 allowlist_functions: Optional[List[str]] = None,
                 allowlist_types: Optional[List[str]] = None,
                 allowlist_vars: Optional[List[str]] = None,
                 include_paths: Optional[List[str]] = None,
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
    for path in (include_paths or []):
        cmd += ["--include-path", path]
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


def _report_check(checked_in: str, generated: str) -> bool:
    """True when the freshly generated file matches the checked-in one."""
    with open(checked_in, encoding="utf-8") as f:
        current = f.read()
    with open(generated, encoding="utf-8") as f:
        fresh = f.read()
    if current == fresh:
        return True
    sys.stderr.writelines(difflib.unified_diff(
        current.splitlines(keepends=True), fresh.splitlines(keepends=True),
        fromfile=f"{checked_in} (checked in)", tofile=f"{checked_in} (regenerated)"))
    return False


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate Rust FFI bindings for rs_rtos (FreeRTOS & Fast_EventGroup)")
    parser.add_argument("--verbose", "-v", action="store_true",
                        help="Show detailed progress")
    parser.add_argument("--check", action="store_true",
                        help="Generate into a temporary directory and compare with the "
                             "checked-in files instead of writing them, exiting non-zero "
                             "when they differ")
    args = parser.parse_args()

    verbose = args.verbose
    pwd = os.path.dirname(os.path.abspath(__file__))
    esprit_root = os.path.realpath(os.path.join(pwd, "..", "..", ".."))
    c_src_dir = os.path.join(esprit_root, "rust", "rust_esprit", "c_interface")
    dest = os.path.join(pwd, "..", "src")

    jobs = [
        # 1. Fast Event Group bindings (C)
        dict(header=os.path.join(c_src_dir, "lnFast_EventGroup_c.h"),
             name="c_fast_event.rs",
             extra_dir=c_src_dir,
             extra_dir2=os.path.join(esprit_root, "FreeRTOS"),
             lang="c"),
        # 2. FreeRTOS bindings (C++)
        dict(header=os.path.join(c_src_dir, "lnFreeRTOS_c.h"),
             name="c_freertos.rs",
             extra_dir=os.path.join(esprit_root, "freertos_config"),
             extra_dir2=os.path.join(esprit_root, "FreeRTOS"),
             lang="c++",
             allowlist_functions=FREERTOS_ALLOW_FUNCTIONS,
             allowlist_types=FREERTOS_ALLOW_TYPES),
    ]

    # --check never touches the source tree: generate elsewhere, then compare.
    out_dir = tempfile.mkdtemp(prefix="rs_rtos_bindgen_") if args.check else dest

    if verbose:
        action = "Checking" if args.check else "Generating"
        print(f"{action} rs_rtos C FFI bindings...")
        print(f"  Output dir: {out_dir}")
        print()

    stale = []
    for job in jobs:
        name = job.pop("name")
        _run_rustgen(output=os.path.join(out_dir, name), verbose=verbose, **job)
        if args.check:
            checked_in = os.path.join(dest, name)
            if _report_check(checked_in, os.path.join(out_dir, name)):
                print(f"  {name}: up to date")
            else:
                print(f"  {name}: OUTDATED", file=sys.stderr)
                stale.append(name)

    if args.check:
        if stale:
            print(f"ERROR: {len(stale)} binding file(s) do not match their headers: "
                  f"{', '.join(stale)}", file=sys.stderr)
            print("Run esprit/rust/rs_rtos/bindgen.sh and commit the result.",
                  file=sys.stderr)
            sys.exit(1)
        if verbose:
            print("\nAll bindings up to date.")
    elif verbose:
        print("\nDone.")


if __name__ == "__main__":
    main()
