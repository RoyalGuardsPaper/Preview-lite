# Preview Lite (TEMPORARY unbuilt version)

Preview Lite is a small, offline-first PDF and image viewer for Linux. It uses Rust, GTK 4, Libadwaita, Poppler, and GdkPixbuf. It opens files directly from local storage and contains no account, telemetry, cloud, or network features.

## Phase 1 features

- Open PDF documents and images through a native file picker or by dropping a local file on the window.
- View PDF pages rendered by Poppler and images supported by the installed GdkPixbuf loaders.
- Browse PDF page thumbnails and navigate with the page controls.
- Zoom in, zoom out, reset to 100%, and fit a page to the viewer.
- Use Ctrl+O, Page Up/Page Down, Space/Backspace, +/−, and 0 shortcuts.
- Scroll large pages and images.

The actual image formats depend on GdkPixbuf loaders installed on the system. This project currently provides viewing only; it does not edit or save documents.

## Screenshot

Screenshot placeholder: capture the application window after launching the first Linux build.

## Build on Linux

Install Rust and the development packages for GTK 4.12 or newer, Libadwaita 1.4 or newer, Poppler GLib, and GdkPixbuf. On Debian or Ubuntu, install Rust with your distribution's Rust packages or rustup, then install `libgtk-4-dev`, `libadwaita-1-dev`, `libpoppler-glib-dev`, and `libgdk-pixbuf-2.0-dev`.

```sh
cargo build --release
cargo run --release
```

The first Cargo build needs the Rust crate dependencies available in its local cache or from crates.io. Runtime viewing and file processing are local and need no network connection.

## Shortcuts

| Action | Shortcut |
| --- | --- |
| Open | Ctrl+O |
| Next page | Page Down or Space |
| Previous page | Page Up or Backspace |
| Zoom in | + or = |
| Zoom out | − |
| Actual size | 0 |

## Privacy

The application has no network client, telemetry, analytics, account, or cloud service. Documents are opened and rendered locally.

## License

This project is licensed under GNU GPL version 3 or (at your option) any later version. See [LICENSE](LICENSE). Third-party dependencies retain their own licenses.

## Limitations

- This is the Phase 1 viewer only. Page editing, markup, OCR, printing, scanning, and image editing are not implemented.
- SVG, HEIF/HEIC, and other less common image formats depend on optional system image loaders.
- It targets Linux with GTK 4 and Poppler GLib installed. A build and runtime check require a Linux GTK development environment.

## AI usage
The app was made almost entirely using Codex, and while I'm not proud of it, and am learning rust as I go, this is how it is for the time being.

## Contributing

Contributions are welcome. Please keep changes small, document user-visible limitations, and build on a Linux system with the dependencies above before proposing changes.
