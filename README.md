# Google Photos Takeout Organizer

A high-performance Rust tool to organize your Google Photos Takeout archive into a clean, chronological folder structure with a modern, thumbnail-optimized web gallery.

![Version: 0.6.1](https://img.shields.io/badge/version-0.6.1-blue.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Rust](https://img.shields.io/badge/Built%20with-Rust-orange)

## 🚀 Features

* **Multiple Archive Support**: Point the tool directly to multiple Google Takeout `.zip` or `.tar.gz` files.
* **Intelligent Date Extraction**: Attempts to find the correct date for each photo/video using JSON metadata, EXIF data, and filename parsing.
* **Chronological Organization**: Sorts files into a `YYYY/MonthName/DD` folder structure (e.g., `2024/January/15`).
* **Modern HTML Gallery with Parallel Thumbnails**: Generates a fast, responsive gallery.
    *   **Automatic Thumbnails**: Pre-generates 400x400 previews for images and videos in parallel (using `Total cores - 1` by default).
    *   **Smart Parallel Video Transcoding**: Can detect incompatible video formats (like **HEVC** from Pixel/iPhone) and generate a web-compatible copy (H.264) for the gallery. **Note:** This process intelligently scales parallel processing based on available system memory (ensuring at least 70% free memory), preventing system instability and preventing memory exhaustion while maximizing speed on capable hardware.
    *   **Modern Aesthetic**: Clean UI with Google-style typography and smooth transitions.
    *   **High Performance**: Uses **Lazy Loading** to ensure smooth scrolling even with thousands of images.
    *   **Interactive Modal**: View media in a large overlay with keyboard navigation.
    *   **Flattened View**: Toggle between hierarchical directory view and a chronological grid.
* **Smart Updates**: Skips already processed files by checking sizes.
* **Fast & Efficient**: Built with Rust for maximum performance and low memory footprint.

## 📋 Requirements

* **FFmpeg (Optional)**: Required for **video thumbnails** and **transcoding HEVC videos**. If FFmpeg is not installed, the tool will warn you, and incompatible videos might not play in some browsers (black screen with audio).
    * **Linux**: `sudo apt install ffmpeg`
    * **macOS**: `brew install ffmpeg`
    * **Windows**: `choco install ffmpeg` or `scoop install ffmpeg`

## 🛠️ Installation

### ⚡ Quick Install (Recommended)

**Linux & macOS**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/raultov/google-photos-takeout-organizer/releases/latest/download/google-photos-takeout-organizer-installer.sh | sh
```

**Windows (PowerShell)**
```powershell
powershell -c "irm https://github.com/raultov/google-photos-takeout-organizer/releases/latest/download/google-photos-takeout-organizer-installer.ps1 | iex"
```

## 📖 Usage

### Organizing from Multiple Archives or Folders

```bash
google-photos-takeout-organizer -i takeout-001.zip takeout-002.zip -o ./MyPhotos
```

### Options

| Option | Short  | Description | Default |
|---|--------|---|---|
| `--input` | `-i`   | Path to source directories or archives (.zip, .tar.gz). **Multiple values allowed.** | |
| `--output` | `-o`   | Path to the destination directory | **Required** |
| `--unknown-dir` | `-u`   | Name of the folder for files with no date | `unknown` |
| `--generate-html` | `-g`  | Generate HTML gallery | `true` |
| `--transcode-videos` | `-t` | Transcode HEVC videos to H.264 (Smart parallel process scaling by memory) | `false` |
| `--threads` | `-j` | Number of parallel thumbnail generation tasks | `Total cores - 1` |

### ⚠️ Performance & Memory Note
Video transcoding is a **heavy operation**. To ensure stability on systems with limited resources (like Mini-PCs with 4GB-8GB RAM), videos are transcoded using a **smart parallel process** that continuously monitors available memory. 

It implements a **gradual "slow-start" throttle mechanism**, starting with just 1 video and scaling up by 1 concurrent task every 3 seconds, provided the system maintains at least 30% available memory. If memory drops below 20%, it will automatically scale down the number of allowed concurrent jobs. This dynamic scaling prevents sudden out-of-memory crashes while significantly speeding up the process on higher-end hardware (only if requested via the `--transcode-videos` flag).

By default, the tool uses `Total cores - 1` for parallel tasks to keep one core free for your desktop environment, ensuring the system remains responsive during heavy processing.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
