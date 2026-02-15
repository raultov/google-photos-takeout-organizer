# Google Photos Takeout Organizer

A high-performance Rust tool to organize your Google Photos Takeout archive into a clean, chronological folder structure.

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Rust](https://img.shields.io/badge/Built%20with-Rust-orange)

## 🚀 Features

* **Intelligent Date Extraction**: Attempts to find the correct date for each photo/video using multiple strategies:
    1.  **JSON Metadata**: Reads the sidecar `.json` files provided by Google Takeout.
    2.  **EXIF Data**: Inspects the file's internal EXIF metadata.
    3.  **Filename Parsing**: Tries to extract dates from filenames (e.g., `IMG_20220101_120000.jpg`).
* **Chronological Organization**: Sorts files into a `YYYY/MM/DD` folder structure.
* **HTML Gallery Generator**: Automatically generates a beautiful, navigable HTML gallery of your organized photos and videos.
    *   **Interactive Modal**: Click photos or videos to view them in a large overlay. Videos can be played directly in the modal.
    *   **Video Thumbnails**: Automatically generates preview thumbnails for videos (requires FFmpeg).
    *   **Slideshow Mode**: Play a slideshow of your media (5-second interval) with date display.
    *   **Flattened View**: Toggle between viewing a month by days or seeing all media for that month in a single grid.
* **Smart Updates**: If you run the tool again, it only copies files that are new or have changed (based on file size), skipping duplicates to save time.
* **New Files Report**: When running incrementally, provides a console report listing exactly which new files were added.
* **Progress Bar**: Shows real-time progress bars for both organization and gallery generation phases, with consistent ETA and status messages.
* **"Unknown" Handling**: Files with no detectable date are moved to a separate `unknown` folder (customizable).
* **Fast & Efficient**: Built with Rust for maximum performance and low memory usage.

## 📋 Requirements

* **FFmpeg (Optional but recommended)**: Required for generating video thumbnails in the HTML gallery. 
    * **Linux**: Install via your package manager (e.g., `sudo apt install ffmpeg`).
    * **macOS**: Install via Homebrew (`brew install ffmpeg`).
    * **Windows**: Install via [Chocolatey](https://chocolatey.org/) (`choco install ffmpeg`), [Scoop](https://scoop.sh/) (`scoop install ffmpeg`), or download the binaries from [ffmpeg.org](https://ffmpeg.org/download.html). **Important**: Ensure `ffmpeg.exe` is in a folder included in your system's **PATH** environment variable.

If FFmpeg is not installed, videos will still be included in the gallery but without preview thumbnails (only a play icon will be shown).

## 🛠️ Installation

### ⚡ Quick Install (Recommended)

The easiest way to install the tool is using the official installer scripts. This will download the pre-compiled binary for your system and add it to your path.

**Linux & macOS**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/raultov/google-photos-takeout-organizer/releases/latest/download/google-photos-takeout-organizer-installer.sh | sh
```

**Windows (PowerShell)**
```powershell
powershell -c "irm https://github.com/raultov/google-photos-takeout-organizer/releases/latest/download/google-photos-takeout-organizer-installer.ps1 | iex"
```

### 📦 Manual Download

You can manually download the executables for Windows, macOS (Intel/Apple Silicon), and Linux from the [Releases Page](https://github.com/raultov/google-photos-takeout-organizer/releases).

### ⚙️ Build from Source

If you prefer to build it yourself, ensure you have Rust & Cargo installed.

```bash
git clone https://github.com/raultov/google-photos-takeout-organizer.git
cd google-photos-takeout-organizer
cargo build --release
```

The binary will be available at `target/release/google-photos-takeout-organizer`.

## 📖 Usage

### Running the installed tool

If you installed via the Quick Install script, you can simply run:

```bash
google-photos-takeout-organizer -i /path/to/takeout -o /path/to/organized_photos
```

The tool will organize your photos and generate an HTML gallery (`collection.html` in the output folder).

### Regenerating HTML Gallery

If you already have an organized folder and just want to regenerate the HTML gallery (e.g., after an update to the tool), you can omit the input directory:

```bash
google-photos-takeout-organizer -o /path/to/organized_photos
```

### Options

| Option | Short  | Description | Default |
|---|--------|---|---|
| `--input` | `-i`   | Path to the source directory (Google Takeout). Optional if regenerating HTML. | |
| `--output` | `-o`   | Path to the destination directory | **Required** |
| `--unknown-dir` | `-u`   | Name of the folder for files with no date | `unknown` |
| `--generate-html` | `-g`  | Generate HTML gallery (set to `false` to disable) | `true` |
| `--help` | `-h`   | Show help message | |

### Examples

**Basic run:**

```bash
# If installed:
google-photos-takeout-organizer -i ./Takeout -o ./MyPhotos

# If running from source:
cargo run --release -- -i ./Takeout -o ./MyPhotos
```

**Custom "unknown" folder:**

```bash
google-photos-takeout-organizer -i ./Takeout -o ./MyPhotos --unknown-dir "unsorted"
```

**Disable HTML generation:**

```bash
google-photos-takeout-organizer -i ./Takeout -o ./MyPhotos --generate-html false
```

### Enable Debug Logging

To see detailed logs about what the tool is doing (e.g., which date source was used for each file):

**Linux/macOS:**
```bash
RUST_LOG=debug google-photos-takeout-organizer -i ...
```

**Windows (PowerShell):**
```powershell
$env:RUST_LOG="debug"; google-photos-takeout-organizer -i ...
```

## 🌍 Cross-Compilation (Raspberry Pi / ARM)

To run this tool on a Raspberry Pi (ARM64) or other architectures manually, the easiest way is to use `cross`.

1. **Install cross:**
    ```bash
    cargo install cross
    ```

2. **Build for ARM64 (Raspberry Pi 3/4/5, 64-bit OS):**
    ```bash
    cross build --target aarch64-unknown-linux-gnu --release
    ```

The compiled binary will be in `target/<target-arch>/release/`.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
