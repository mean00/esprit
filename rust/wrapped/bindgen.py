#!/usr/bin/env python3
"""
Bindgen wrapper for esprit/rust/wrapped/ C FFI bindings.

Generates Rust FFI bindings (src/ffi.rs) for all wrapped backend crates:
  - rs_gpio_wrapped  (from lnGPIO_c.h)
  - rs_i2c_wrapped   (from lnI2C_c.h)
  - rs_spi_wrapped   (from lnSPI_c.h)
  - rs_uart_wrapped  (from lnSerial_c.h)
  - rs_exti_wrapped  (from lnExti_c.h)
  - rs_timer_wrapped (from lnTimer_c.h)
  - rs_adc_wrapped   (from lnTiming_adc_c.h)

Usage:
    ./bindgen.py               # generate bindings for all wrapped crates
    ./bindgen.py rs_gpio       # generate only rs_gpio_wrapped
    ./bindgen.py -v            # verbose output
"""

import argparse
import os
import subprocess
import sys
from typing import List, Optional

_CMAKE_DIR = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", "..", "cmake"))
_RUSTGEN_PY = os.path.join(_CMAKE_DIR, "rustgen.py")


def _run_rustgen(header: str, output: str, extra_dir: str = "",
                 extra_dir2: str = "", blocklist: bool = False,
                 blocklist_items: Optional[List[str]] = None,
                 allowlist_functions: Optional[List[str]] = None,
                 allowlist_types: Optional[List[str]] = None,
                 allowlist_vars: Optional[List[str]] = None,
                 raw_lines: Optional[List[str]] = None,
                 lang: str = "c++", verbose: bool = False,
                 reference_mcu: bool = True) -> None:
    """Run rustgen.py to generate a single binding file."""
    out_name = os.path.basename(os.path.dirname(os.path.dirname(output)))
    print(f"  {out_name} -> {os.path.basename(output)}")

    cmd = [sys.executable, _RUSTGEN_PY,
           "--header", header,
           "--output", output,
           "--lang", lang,
           "--extra-dir", extra_dir]
    if extra_dir2:
        cmd += ["--extra-dir2", extra_dir2]
    header_in = os.path.join(os.path.dirname(os.path.abspath(__file__)), "header.rs.in")
    if os.path.isfile(header_in):
        cmd += ["--header-in", header_in]
    if reference_mcu:
        ln_dir = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", ".."))
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
    for rl in (raw_lines or []):
        cmd += ["--raw-line", rl]
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
        description="Generate Rust FFI bindings for wrapped backend crates")
    parser.add_argument("crates", nargs="*",
                        help="Specific crates to regenerate (e.g. rs_gpio, uart). Default: all.")
    parser.add_argument("--verbose", "-v", action="store_true",
                        help="Show detailed progress")
    args = parser.parse_args()

    verbose = args.verbose
    target_filter = [c.lower() for c in args.crates]

    def should_run(name: str) -> bool:
        if not target_filter:
            return True
        return any(t in name.lower() for t in target_filter)

    pwd = os.path.dirname(os.path.abspath(__file__))
    esprit_root = os.path.realpath(os.path.join(pwd, "..", ".."))
    c_src_dir = os.path.join(esprit_root, "rust", "rust_esprit", "c_interface")

    if verbose:
        print("Generating wrapped crates C FFI bindings...")
        print()

    # 1. rs_gpio_wrapped
    if should_run("gpio"):
        _run_rustgen(
            header=os.path.join(c_src_dir, "lnGPIO_c.h"),
            output=os.path.join(pwd, "rs_gpio_wrapped", "src", "ffi.rs"),
            extra_dir=os.path.join(esprit_root, "mcus", "common_bluepill", "include"),
            lang="c++",
            blocklist_items=["lnPin"],
            raw_lines=["pub type lnPin = cty::c_int;"],
            verbose=verbose,
        )

    # 2. rs_i2c_wrapped
    if should_run("i2c"):
        _run_rustgen(
            header=os.path.join(c_src_dir, "lnI2C_c.h"),
            output=os.path.join(pwd, "rs_i2c_wrapped", "src", "ffi.rs"),
            extra_dir=c_src_dir,
            lang="c",
            raw_lines=["unsafe extern \"C\" { pub fn lni2c_set_dma_mode(ptr: *mut ln_i2c_c, enable: bool); }"],
            verbose=verbose,
        )

    # 3. rs_spi_wrapped
    if should_run("spi"):
        _run_rustgen(
            header=os.path.join(c_src_dir, "lnSPI_c.h"),
            output=os.path.join(pwd, "rs_spi_wrapped", "src", "ffi.rs"),
            extra_dir=c_src_dir,
            lang="c++",
            raw_lines=["unsafe extern \"C\" { pub fn lnspi_set_dma_mode(ptr: *mut ln_spi_c, enable: bool); }"],
            verbose=verbose,
        )

    # 4. rs_uart_wrapped
    if should_run("uart"):
        _run_rustgen(
            header=os.path.join(c_src_dir, "lnSerial_c.h"),
            output=os.path.join(pwd, "rs_uart_wrapped", "src", "ffi.rs"),
            extra_dir=c_src_dir,
            lang="c",
            verbose=verbose,
        )

    # 5. rs_exti_wrapped
    if should_run("exti"):
        exti_blocklist = [
            "lnPin",
            "lnExtiAttachInterrupt", "lnExtiDetachInterrupt",
            "lnExtiEnableInterrupt", "lnExtiDisableInterrupt",
        ]
        _run_rustgen(
            header=os.path.join(c_src_dir, "lnExti_c.h"),
            output=os.path.join(pwd, "rs_exti_wrapped", "src", "ffi.rs"),
            extra_dir=c_src_dir,
            lang="c++",
            blocklist_items=exti_blocklist,
            raw_lines=["pub type lnPin = cty::c_int;"],
            verbose=verbose,
        )

    # 6. rs_timer_wrapped
    if should_run("timer"):
        timer_fns = [
            "ln_timer_.*", "lnDelay_C",
        ]
        timer_types = [
            "ln_timer_c",
        ]
        timer_raw_lines = [
            "pub type lnPin = cty::c_int;",
            "#[repr(C)]",
            "#[derive(Debug, Copy, Clone)]",
            "pub struct ln_delay_timer_c {",
            "    pub dummy: *mut cty::c_void,",
            "}",
            "pub type DelayTimerCallback = Option<unsafe extern \"C\" fn(cookie: *mut cty::c_void)>;",
            "unsafe extern \"C\" {",
            "    pub fn ln_delay_timer_create(timer: cty::c_int, channel: cty::c_int) -> *mut ln_delay_timer_c;",
            "    pub fn ln_delay_timer_delete(timer: *mut ln_delay_timer_c);",
            "    pub fn ln_delay_timer_arm(timer: *mut ln_delay_timer_c, delay_us: cty::c_int);",
            "    pub fn ln_delay_timer_set_interrupt(",
            "        timer: *mut ln_delay_timer_c,",
            "        handler: DelayTimerCallback,",
            "        cookie: *mut cty::c_void,",
            "    );",
            "    pub fn ln_delay_timer_enable_interrupt(timer: *mut ln_delay_timer_c);",
            "    pub fn ln_delay_timer_disable_interrupt(timer: *mut ln_delay_timer_c);",
            "}",
        ]
        _run_rustgen(
            header=os.path.join(c_src_dir, "lnTimer_c.h"),
            output=os.path.join(pwd, "rs_timer_wrapped", "src", "ffi.rs"),
            extra_dir=os.path.join(esprit_root, "mcus", "common_bluepill", "include"),
            lang="c++",
            blocklist_items=["lnPin"],
            allowlist_functions=timer_fns,
            allowlist_types=timer_types,
            raw_lines=timer_raw_lines,
            verbose=verbose,
        )

    # 7. rs_adc_wrapped
    if should_run("adc"):
        adc_raw_lines = [
            "pub type lnPin = cty::c_int;",
            "unsafe extern \"C\" {",
            "    pub fn ln_simple_adc_create(instance: u32, pin: u32) -> *mut cty::c_void;",
            "    pub fn ln_simple_adc_set_smpt(adc: *mut cty::c_void, smpt: u32);",
            "    pub fn ln_simple_adc_read(adc: *mut cty::c_void) -> i32;",
            "    pub fn ln_simple_adc_destroy(adc: *mut cty::c_void);",
            "}",
        ]
        _run_rustgen(
            header=os.path.join(c_src_dir, "lnTiming_adc_c.h"),
            output=os.path.join(pwd, "rs_adc_wrapped", "src", "ffi.rs"),
            extra_dir=c_src_dir,
            lang="c++",
            blocklist_items=["lnPin"],
            raw_lines=adc_raw_lines,
            verbose=verbose,
        )

    if verbose:
        print()
        print("Done.")


if __name__ == "__main__":
    main()
