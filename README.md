# Matterhorn

A lenient front matter parsing crate that supports files prefixed with YAML, JSON, and TOML front matter.
The type of front matter is detected automatically.

The order of input keys is retained, and parsing is lenient where possible.
Notably, duplicate keys are supported in YAML front matter.

All parsed front matter is returned as a `serde_json` Value.

## Usage

```bash
cargo add matterhorn
```

```rust
const YAML_SOURCE_FILE: &str = r#"
---
title: Hello World
order: 12
---
# Main Title

Cras mattis consectetur purus sit amet fermentum.
"#;

const TOML_SOURCE_FILE: &str = r#"
+++
title = "Hello World"
order = 12
+++
# Main Title

Cras mattis consectetur purus sit amet fermentum.
"#;

const JSON_SOURCE_FILE: &str = r#"
{
  "title": "Hello World",
  "order": 12
}
# Main Title

Cras mattis consectetur purus sit amet fermentum.
"#;

fn main() {
    let document = matterhorn::parse_document(YAML_SOURCE_FILE).expect("Input should be valid");

    println!("{:#?}", document.front_matter);
    // Returns:
    // serde_json::Value::Object {
    //     "title": serde_json::Value::String("Hello World"),
    //     "order": serde_json::Value::Number(12),
    // }

    println!("{:#?}", document.content);
    // Returns:
    // "# Main Title\n\nCras mattis consectetur purus sit amet fermentum.\n"

    assert_eq!(matterhorn::parse_document(YAML_SOURCE_FILE), matterhorn::parse_document(TOML_SOURCE_FILE));
    assert_eq!(matterhorn::parse_document(TOML_SOURCE_FILE), matterhorn::parse_document(JSON_SOURCE_FILE));
}
```
