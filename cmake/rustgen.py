#!/usr/bin/env python3
"""
Bindgen wrapper that generates a single Rust FFI binding file.

Replaces rustgen.sh / rustgen_c.sh / rustgen_lang.sh with a single Python
script that:
  - Locates the bindgen binary
  - Resolves the platform clang path via cmake
  - Prepares a temporary header (with uint32_t workaround for ARM)
  - Runs bindgen with the correct flags
  - Prepends header.rs.in and appends tail.rs.in

Usage:
    rustgen.py --header <file.h> --output <file.rs> --lang c++ \\
               --extra-dir <dir> [--extra-dir2 <dir>] [--blocklist] [--verbose]
"""

import argparse
import os
import shutil
import subprocess
import sys
import tempfile


def find_bindgen() -> str:
    """Locate the bindgen binary."""
    candidates = [
        os.path.expanduser("~/.cargo/bin/bindgen"),
        "/usr/bin/bindgen",
    ]
    for c in candidates:
        if os.path.isfile(c):
            return c
    # Fall back to PATH lookup
    which = shutil.which("bindgen")
    if which:
        return which
    print("ERROR: bindgen not found. Install with: cargo install bindgen-cli",
          file=sys.stderr)
    sys.exit(1)


def get_platform_clang_path(ln_dir: str) -> str:
    """Resolve PLATFORM_CLANG_PATH via cmake -P toolchain_env.cmake."""
    cmake_file = os.path.join(ln_dir, "cmake", "toolchain_env.cmake")
    if not os.path.isfile(cmake_file):
        print(f"ERROR: toolchain_env.cmake not found at {cmake_file}",
              file=sys.stderr)
        sys.exit(1)
    result = subprocess.run(
        ["cmake", f"-DUSE_CLANG=1", f"-DHOME={ln_dir}", "-P", cmake_file],
        capture_output=True, text=True,
    )
    if result.returncode != 0:
        print(f"ERROR: cmake toolchain_env.cmake failed:\n{result.stderr}",
              file=sys.stderr)
        sys.exit(1)
    # cmake's message(NOTICE ...) goes to stderr, not stdout
    path = result.stderr.strip()
    if not path:
        print("ERROR: cmake toolchain_env.cmake returned empty path",
              file=sys.stderr)
        sys.exit(1)
    return path


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate a single Rust FFI binding file via bindgen")
    parser.add_argument("--header", required=True, help="Input C/C++ header file")
    parser.add_argument("--output", required=True, help="Output Rust file")
    parser.add_argument("--lang", required=True, choices=["c", "c++"],
                        help="Language mode for clang")
    parser.add_argument("--extra-dir", required=True,
                        help="Extra include directory")
    parser.add_argument("--extra-dir2", default="",
                        help="Second extra include directory")
    parser.add_argument("--blocklist", action="store_true",
                        help="Blocklist lnPin type and inject use crate::pin_types::lnPin")
    parser.add_argument("--verbose", "-v", action="store_true",
                        help="Show detailed progress")
    args = parser.parse_args()

    header = os.path.abspath(args.header)
    output = os.path.abspath(args.output)
    lang = args.lang
    extra_dir = os.path.abspath(args.extra_dir) if args.extra_dir else ""
    extra_dir2 = os.path.abspath(args.extra_dir2) if args.extra_dir2 else ""
    blocklist = args.blocklist
    verbose = args.verbose

    # Locate bindgen
    bindgen = find_bindgen()
    if verbose:
        print(f"  bindgen: {bindgen}")

    # Determine LN (esprit root) from this script's location
    script_dir = os.path.dirname(os.path.abspath(__file__))
    ln_dir = os.path.realpath(os.path.join(script_dir, ".."))

    # Resolve platform clang path
    platform_clang_path = get_platform_clang_path(ln_dir)
    if verbose:
        print(f"  clang:   {platform_clang_path}")

    # Update PATH
    env = os.environ.copy()
    env["PATH"] = f"{platform_clang_path}:{env.get('PATH', '')}"

    # header.rs.in and tail.rs.in are always taken from the c_interface/ directory
    # (the same directory as the C wrapper headers), not from src/.
    # The src/ tail.rs.in contains lnFastEventGroup Send/Sync impls which are
    # specific to the fast event group bindings and must not be appended to
    # other binding files.
    c_interface_dir = os.path.join(ln_dir, "rust", "rust_esprit", "c_interface")
    header_in = os.path.join(c_interface_dir, "header.rs.in")
    tail_in = os.path.join(c_interface_dir, "tail.rs.in")
    if not os.path.isfile(header_in):
        print(f"ERROR: header.rs.in not found in {c_interface_dir}",
              file=sys.stderr)
        sys.exit(1)

    # Prepare temporary header with uint32_t workaround for ARM
    with open(header, "r") as f:
        header_content = f.read()

    tmp_header = header + ".tmp"
    try:
        with open(tmp_header, "w") as f:
            f.write("#ifdef __ARM_ARCH\n")
            f.write("   #undef uint32_t\n")
            f.write("   #define _UINT32_T_DECLARED\n")
            f.write("   #define uint32_t unsigned int\n")
            f.write("#endif\n")
            f.write(header_content)

        # Build bindgen arguments
        bindgen_args = [
            bindgen,
            "--rust-edition", "2024",
            "--rust-target", "1.85",
            "--use-core",
            "--no-doc-comments",
            "--no-layout-tests",
            tmp_header,
            "--ctypes-prefix", "cty",
            "--rustified-enum", "ln.*",
        ]

        if blocklist:
            bindgen_args += [
                "--blocklist-type", "lnPin",
                "--raw-line", "pub use crate::pin_types::lnPin;",
            ]

        tmp_output = output + ".tmp"
        bindgen_args += ["-o", tmp_output]

        # Clang arguments (after --)
        clang_args = [
            "-x", lang,
            "-DLN_ARCH=LN_ARCH_ARM",
            "-funsigned-char",
            f"-I{extra_dir}",
        ]
        if extra_dir2:
            clang_args.append(f"-I{extra_dir2}")
        clang_args += [
            f"-I{ln_dir}",
            f"-I{ln_dir}/include/",
            f"-I{ln_dir}/arduinoLayer/include/",
            f"-I{ln_dir}/FreeRTOS/include/",
            f"-I{ln_dir}/mcus/arm_gd32fx/boards/bluepill/",
            f"-I{ln_dir}/mcus/arm_gd32fx/include/",
            f"-I{ln_dir}/mcus/common_bluepill/",
            f"-I{ln_dir}/legacy/boards/bluepill/",
            f"-I{ln_dir}/FreeRTOS/portable/GCC/ARM_CM3/",
            "-target", "thumbv7m-none-eabi",
            f"-I{platform_clang_path}/../lib/clang-runtimes/arm-none-eabi/armv7m_soft_nofp/include/",
        ]

        bindgen_args.append("--")
        bindgen_args += clang_args

        cmdline = ' '.join(bindgen_args)
        if verbose:
            print(f"  running: {cmdline}")

        # Run bindgen
        result = subprocess.run(
            bindgen_args,
            capture_output=True, text=True,
            env=env,
        )
        if result.returncode != 0:
            print(f"ERROR: bindgen failed for {header}", file=sys.stderr)
            print(f"Command: {cmdline}", file=sys.stderr)
            print(result.stderr, file=sys.stderr)
            sys.exit(1)

        # Concatenate header.rs.in + bindgen output + tail.rs.in
        with open(output, "w") as f_out:
            with open(header_in, "r") as f_in:
                f_out.write(f_in.read())
            with open(tmp_output, "r") as f_in:
                f_out.write(f_in.read())
            if os.path.isfile(tail_in):
                with open(tail_in, "r") as f_in:
                    f_out.write(f_in.read())

    finally:
        # Clean up temp files
        for tmp in [tmp_header, tmp_output]:
            if os.path.isfile(tmp):
                os.unlink(tmp)

    if verbose:
        print(f"  wrote: {output}")


if __name__ == "__main__":
    main()