# Google Photos Takeout Organizer

A high-performance Rust tool to organize your Google Photos Takeout archive into a clean, chronological folder structure.

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Rust](https://img.shields.io/badge/Built%20with-Rust-orange)

## 🚀 Features

*   **Intelligent Date Extraction**: Attempts to find the correct date for each photo/video using multiple strategies:
    1.  **JSON Metadata**: Reads the sidecar `.json` files provided by Google Takeout.
    2.  **EXIF Data**: Inspects the file's internal EXIF metadata.
    3.  **Filename Parsing**: Tries to extract dates from filenames (e.g., `IMG_20220101_120000.jpg`).
*   **Chronological Organization**: Sorts files into a `YYYY/MM/DD` folder structure.
*   **Smart Updates**: If you run the tool again, it only copies files that are new or have changed (based on file size), skipping duplicates to save time.
*   **"Unknown" Handling**: Files with no detectable date are moved to a separate `unknown` folder (customizable).
*   **Fast & Efficient**: Built with Rust for maximum performance and low memory usage.

## 🛠️ Installation

### Prerequisites

*   [Rust & Cargo](https://www.rust-lang.org/tools/install) installed on your system.

### Build from Source

```bash
git clone https://github.com/raultov/google-photos-takeout-organizer.git
cd google-photos-takeout-organizer
cargo build --release
```

The binary will be available at `target/release/google-photos-takeout-cli`.

## 📖 Usage

Basic usage requires specifying the input directory (your Google Takeout folder) and the output directory (where you want the organized photos).

```bash
cargo run --release -- -i /path/to/takeout -o /path/to/organized_photos
```

### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--input` | `-i`  | Path to the source directory (Google Takeout) | **Required** |
| `--output` | `-o`  | Path to the destination directory | **Required** |
| `--unknown-dir` | `-u`  | Name of the folder for files with no date | `unknown` |
| `--help` | `-h`  | Show help message | |

### Examples

**Basic run:**
```bash
./google-photos-takeout-cli -i ./Takeout -o ./MyPhotos
```

**Custom "unknown" folder:**
```bash
./google-photos-takeout-cli -i ./Takeout -o ./MyPhotos --unknown-dir "unsorted"
```

**Enable Debug Logging:**
To see detailed logs about what the tool is doing (e.g., which date source was used for each file):

*   **Linux/macOS:**
    ```bash
    RUST_LOG=debug ./google-photos-takeout-cli -i ...
    ```
*   **Windows (PowerShell):**
    ```powershell
    $env:RUST_LOG="debug"; ./google-photos-takeout-cli.exe -i ...
    ```

## 🌍 Cross-Compilation (e.g., Raspberry Pi)

To run this tool on a Raspberry Pi (ARM64) or other architectures, the easiest way is to use [`cross`](https://github.com/cross-rs/cross).

1.  **Install `cross`:**
    ```bash
    cargo install cross
    ```

2.  **Build for ARM64 (Raspberry Pi 3/4/5, 64-bit OS):**
    ```bash
    cross build --target aarch64-unknown-linux-gnu --release
    ```

3.  **Build for ARMv7 (Raspberry Pi 2/3/4, 32-bit OS):**
    ```bash
    cross build --target armv7-unknown-linux-gnueabihf --release
    ```

The compiled binary will be in `target/<target-arch>/release/`. simply copy it to your device and run it.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
