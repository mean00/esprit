#!/usr/bin/env python3
"""
Bindgen wrapper for rust_esprit C FFI bindings.

Generates Rust FFI bindings from C/C++ wrapper headers using bindgen.
Supports blocklisting types (e.g. lnPin) to avoid duplicate definitions.

Usage:
    ./bindgen.py                  # generate all bindings (quiet)
    ./bindgen.py --verbose        # generate all bindings (verbose)
"""

import argparse
import os
import subprocess
import sys

# Path to the cmake helper scripts
_CMAKE_DIR = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", "..", "..", "cmake"))
_RUSTGEN_PY = os.path.join(_CMAKE_DIR, "rustgen.py")


def _run_rustgen(header: str, output: str, extra_dir: str = "",
                 extra_dir2: str = "", blocklist: bool = False,
                 lang: str = "c++", verbose: bool = False) -> None:
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


def gen_cpp(header: str, output: str, extra_dir: str = "",
            extra_dir2: str = "", blocklist: bool = False,
            verbose: bool = False) -> None:
    """Generate C++ bindings for a header."""
    if verbose:
        print(f"  [C++] {header}")
    _run_rustgen(header, output, extra_dir=extra_dir, extra_dir2=extra_dir2,
                 blocklist=blocklist, lang="c++", verbose=verbose)


def gen_c(header: str, output: str, extra_dir: str = "",
          extra_dir2: str = "", blocklist: bool = False,
          verbose: bool = False) -> None:
    """Generate C bindings for a header."""
    if verbose:
        print(f"  [C]   {header}")
    _run_rustgen(header, output, extra_dir=extra_dir, extra_dir2=extra_dir2,
                 blocklist=blocklist, lang="c", verbose=verbose)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate Rust FFI bindings for rust_esprit C/C++ headers")
    parser.add_argument("--verbose", "-v", action="store_true",
                        help="Show detailed progress")
    args = parser.parse_args()

    verbose = args.verbose
    pwd = os.path.dirname(os.path.abspath(__file__))
    dest = os.path.join(pwd, "..", "src", "c_api")

    if verbose:
        print("Generating rust_esprit C FFI bindings...")
        print(f"  Output dir: {dest}")
        print()

    # --- GPIO bindings (one per platform) ---
    # Bluepill / GD32 / STM32
    _run_rustgen(
        header=os.path.join(pwd, "lnGPIO_c.h"),
        output=os.path.join(dest, "rn_gpio_bp_c.rs"),
        extra_dir=os.path.join(pwd, "..", "..", "..", "mcus", "common_bluepill", "include"),
        lang="c++", verbose=verbose,
    )
    # RP2040
    _run_rustgen(
        header=os.path.join(pwd, "lnGPIO_c.h"),
        output=os.path.join(dest, "rn_gpio_rp2040_c.rs"),
        extra_dir=os.path.join(pwd, "..", "..", "..", "mcus", "arm_rp2040", "include"),
        lang="c++", verbose=verbose,
    )
    # ESP32
    _run_rustgen(
        header=os.path.join(pwd, "lnGPIO_c.h"),
        output=os.path.join(dest, "rn_gpio_esp32_c.rs"),
        extra_dir=os.path.join(pwd, "..", "..", "..", "mcus", "riscv_esp32", "include"),
        lang="c++", verbose=verbose,
    )

    # --- Bindings that need lnPin blocklist ---
    # These headers only forward-declare `enum lnPin : int;` without defining
    # the enum values.  Bindgen would generate a stub enum that conflicts with
    # the real one from the GPIO bindings.  We blocklist lnPin and inject a
    # `use crate::pin_types::lnPin;` instead.
    gen_cpp(
        header=os.path.join(pwd, "lnTiming_adc_c.h"),
        output=os.path.join(dest, "rn_timing_adc_c.rs"),
        extra_dir=pwd, blocklist=True, verbose=verbose,
    )
    gen_cpp(
        header=os.path.join(pwd, "lnTimer_c.h"),
        output=os.path.join(dest, "rn_timer_c.rs"),
        extra_dir=os.path.join(pwd, "..", "..", "..", "mcus", "common_bluepill", "include"),
        blocklist=True, verbose=verbose,
    )
    gen_cpp(
        header=os.path.join(pwd, "lnExti_c.h"),
        output=os.path.join(dest, "rn_exti_c.rs"),
        extra_dir=pwd, blocklist=True, verbose=verbose,
    )

    # --- Other bindings (no blocklist needed) ---
    gen_c(
        header=os.path.join(pwd, "lnI2C_c.h"),
        output=os.path.join(dest, "rn_i2c_c.rs"),
        extra_dir=pwd, verbose=verbose,
    )
    gen_cpp(
        header=os.path.join(pwd, "lnSPI_c.h"),
        output=os.path.join(dest, "rn_spi_c.rs"),
        extra_dir=pwd, verbose=verbose,
    )
    gen_c(
        header=os.path.join(pwd, "lnCDC_c.h"),
        output=os.path.join(dest, "rn_cdc_c.rs"),
        extra_dir=pwd, verbose=verbose,
    )
    gen_c(
        header=os.path.join(pwd, "lnUSB_c.h"),
        output=os.path.join(dest, "rn_usb_c.rs"),
        extra_dir=pwd, verbose=verbose,
    )
    gen_c(
        header=os.path.join(pwd, "lnFast_EventGroup_c.h"),
        output=os.path.join(dest, "rn_fast_event_c.rs"),
        extra_dir=pwd,
        extra_dir2=os.path.join(pwd, "..", "..", "..", "FreeRTOS"),
        verbose=verbose,
    )

    # FreeRTOS bindings
    if verbose:
        print("  [C]   lnFreeRTOS_c.h")
    _run_rustgen(
        header=os.path.join(pwd, "lnFreeRTOS_c.h"),
        output=os.path.join(dest, "rn_freertos_c.rs"),
        extra_dir=os.path.join(pwd, "..", "..", "..", "freertos_config"),
        extra_dir2=os.path.join(pwd, "..", "..", "..", "FreeRTOS"),
        lang="c++", verbose=verbose,
    )

    # Debug logger
    gen_cpp(
        header=os.path.join(pwd, "..", "..", "..", "include", "lnDebug.h"),
        output=os.path.join(dest, "rn_debug_c.rs"),
        extra_dir=pwd, verbose=verbose,
    )

    # Serial
    gen_c(
        header=os.path.join(pwd, "lnSerial_c.h"),
        output=os.path.join(dest, "rn_serial_c.rs"),
        extra_dir=pwd, verbose=verbose,
    )

    if verbose:
        print()
        print("Done.")


if __name__ == "__main__":
    main()