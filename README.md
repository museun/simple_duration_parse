# simple_duration_parse

a simple, quick method for parsing a human-readable duration into different types, or just as seconds

# Optional features
>time

enabling the `time` feature will allow parsing into a _time v0.2_ `Duration`

# Format
| suffix | description |
| --- | --- |
| d | days |
| h | hours |
| m | minutes |
| s | seconds |

# Example

```rust
use simple_duration_parse::{DurationParser as _, parse_secs};
use std::time::Duration;

let tests = &[
    ("1s", 1),
    ("1m", 60),
    ("1h", 60 * 60),
    ("1h 1m 1s", (60 * 60) + 60 + 1),
    ("1h 1s", (60 * 60) + 1),
    ("30m 59s", (30 * 60) + 59),
    ("7d", (60 * 60 * 24 * 7)),
    ("3d 5m", (60 * 60 * 24 * 3) + 5 * 60),
    ("1s foobar", 1),
];

for (input, expected) in tests {
    assert_eq!(parse_secs(&input).unwrap(), *expected);
}

for (input, expected) in tests {
    let duration = Duration::parse_human_duration(input).unwrap();
    assert_eq!(Duration::from_secs(*expected), duration);
}
```

License: 0BSD
