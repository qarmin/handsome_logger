## 0.10.0 - 22.10.2026

- Added `RotatingFile` - writer with panic-free rotation by size, time or both (`RotationType`), timestamp or numbered file names (`Naming`), `max_files` retention (10 by default), `rotate_on_startup` and local/UTC time zone choice
- Added `WriteLogger::init_rotating` and `WriteLogger::new_rotating`
- Added `ConfigBuilder::for_file()`, `init_with_file` and `CombinedLogger::builder()` (with `terminal`, `terminal_with`, `file` and `fallback_to_terminal`), to log both to terminal and rotated file in few lines
- Added `ConfigBuilder::set_allowed_modules` and `ConfigBuilder::set_ignored_modules`
- **Breaking**: `set_write_once` was renamed to `set_single_write` and is now enabled by default
- Rewritten rotating example to not require `file-rotate` crate

## 0.9.2 - 19.02.2026

- Add `timezone_file_access` feature, to be able to remove file system access for timezone data, which is problematic in miri.
- Added ability to print simple log messages without any time, to avoid accessing timezone data at all
- Increased minimum rust version to 1.88

## 0.9.1 - 11.03.2025

- Show by default colors

## 0.9.0 - 11.03.2025

- Updated tz-rs
- Added enum with predefinied text formats
- Decreased number of code, needed by typical setup
- Increased minimum rust version to 1.81

## 0.8.0 - 29.09.2023

- By default local time offset is used instead of UTC
- Chrono dependency is completely removed and tz-rs is used instead

## 0.7.1 - 27.09.2023

- Added ability to use `handsome_logger::init();` for really simple initialization
- Added ability to use non-lowercased log levels in `RUST_LOG` environment variable

## 0.7.0 - 19.09.2023

- Added ability to set log level by environment variable

## 0.6.0 - 15.09.2023

- Added by default showing microseconds - it is quite usable when testing quite fast operations
- Increased minimum rust version from 1.65 to 1.67

## 0.5.0 - 14.07.2023

- Added custom user function formatting

## 0.4.0 - 09.07.2023

- Added ability to filter messages by content

## 0.3.0 - 04.07.2023

- Added example with file rotating
- Added ability to write all items at once to writer object(useful for file rotating)
- New type of items - `[_thread_name]` and `[_process_id]`
- Renamed `[_thread]` to `[_thread_id]`

## 0.2.0 - 02.07.2023

- Added examples
- Fixed problem with additional and missing new lines in the output
- Added support for using different text and background colors

## 0.1.0 - 02.07.2023

- Initial Release
