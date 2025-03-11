## 0.9.9 - 11.03.2025

- Updated tz-rs
- Added enum with predefinied text formats
- Decreased number of code, needed by typical setup

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
