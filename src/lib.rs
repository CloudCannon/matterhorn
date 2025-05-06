use std::ops::Add;

use conversion::yaml_to_json;
use saphyr::LoadableYamlNode;
use thiserror::Error;

mod conversion;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MatterhornError {
    #[error("{msg}")]
    Custom { msg: String },
}

#[derive(Debug, PartialEq, Eq)]
pub struct MatterhornDocument<'d> {
    pub front_matter: serde_json::Value,
    pub content: &'d str,
}

/// Parse a full document into a front matter object and text content,
/// with auto-detection for the front matter type.
pub fn parse_document<'d>(
    original_content: &'d str,
) -> Result<MatterhornDocument<'d>, MatterhornError> {
    let Some(document_start) = original_content.find(|c: char| !c.is_whitespace()) else {
        return Ok(MatterhornDocument {
            front_matter: serde_json::Value::Null,
            content: original_content,
        });
    };

    let relative_document_content = &original_content[document_start..];
    let (separator, should_parse_separator) = if relative_document_content.starts_with("---") {
        ("---", false)
    } else if relative_document_content.starts_with("+++") {
        ("+++", false)
    } else if relative_document_content.starts_with('{') {
        ("}", true)
    } else {
        return Ok(MatterhornDocument {
            front_matter: serde_json::Value::Null,
            content: original_content,
        });
    };

    let fm_start = if should_parse_separator {
        document_start
    } else {
        document_start + separator.len()
    };

    for (pre_fm_end, _) in original_content
        .match_indices(separator)
        .filter(|(i, _)| *i > fm_start)
    {
        let post_fm_end = pre_fm_end + separator.len();
        let potential_front_matter = if should_parse_separator {
            &original_content[fm_start..post_fm_end]
        } else {
            &original_content[fm_start..pre_fm_end]
        };

        let content_start = || {
            let relative_next_newline = &original_content[post_fm_end..]
                .find('\n')
                .unwrap_or_default()
                .add(1);
            let document_start = post_fm_end + relative_next_newline;
            if document_start > original_content.len() {
                original_content.len()
            } else {
                document_start
            }
        };

        if let Ok(value) = parse_toml(potential_front_matter) {
            if value.as_object().is_some_and(|obj| !obj.is_empty()) {
                return Ok(MatterhornDocument {
                    front_matter: value,
                    content: &original_content[content_start()..],
                });
            }
        }

        if let Ok(value) = parse_yaml(potential_front_matter) {
            if value.as_object().is_some_and(|obj| !obj.is_empty()) {
                return Ok(MatterhornDocument {
                    front_matter: value,
                    content: &original_content[content_start()..],
                });
            }
        }
    }

    Err(MatterhornError::Custom {
        msg: "Failed to parse document".to_string(),
    })
}

/// Parse JSON.
/// NB: Maintains source order.
pub fn parse_json(content: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(content).map_err(|err| err.to_string())
}

/// Parse TOML.
pub fn parse_toml(content: &str) -> Result<serde_json::Value, String> {
    toml::from_str(content).map_err(|err| err.to_string())
}

/// Parse YAML.
/// NB: Maintains source order.
/// NB: Allows duplicate keys.
pub fn parse_yaml(content: &str) -> Result<serde_json::Value, String> {
    let mut yaml = saphyr::Yaml::load_from_str(content).map_err(|err| err.to_string())?;
    yaml.retain(|val| !val.is_null());

    if yaml.len() != 1 {
        return Err("Multiple yaml documents are unsupported".to_string());
    }

    let yaml = yaml
        .pop()
        .expect("Yaml file should contain exactly one document");

    yaml_to_json(yaml).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::*;

    #[test]
    fn plain_file() {
        let input = r#"
# Hello World!

Donec ullamcorper nulla non metus auctor fringilla. Maecenas faucibus mollis interdum.
"#;
        let parsed = parse_document(input);

        assert_eq!(
            parsed,
            Ok(MatterhornDocument {
                front_matter: Value::Null,
                content: input
            })
        );
    }

    #[test]
    fn yaml_fm() {
        let input = r#"
---
title: Hello World
list:
  - 1
  - "two"
  - key: value
    value: " Text --- with --- lots --- of --- these "
---

Cras mattis consectetur purus sit amet fermentum. Donec sed odio dui.

> Morbi leo risus, porta ac consectetur ac, vestibulum at eros.
"#;
        let parsed = parse_document(input);

        assert_eq!(
            parsed,
            Ok(MatterhornDocument {
                front_matter: json!({
                    "title": "Hello World",
                    "list": [1, "two", {
                        "key": "value",
                        "value": " Text --- with --- lots --- of --- these "
                    }]
                }),
                content: r#"
Cras mattis consectetur purus sit amet fermentum. Donec sed odio dui.

> Morbi leo risus, porta ac consectetur ac, vestibulum at eros.
"#
            })
        );
    }

    #[test]
    fn toml_fm() {
        let input = r#"
+++
title = "Hello World"

items = [1, "two"]

[outer]
inner.name = "Justo +++ Parturient"
+++

Cras mattis consectetur purus sit amet fermentum. Donec sed odio dui.

> Morbi leo risus, porta ac consectetur ac, vestibulum at eros.
"#;
        let parsed = parse_document(input);

        assert_eq!(
            parsed,
            Ok(MatterhornDocument {
                front_matter: json!({
                    "title": "Hello World",
                    "items": [1, "two"],
                    "outer": {
                        "inner": {
                            "name": "Justo +++ Parturient"
                        }
                    }
                }),
                content: r#"
Cras mattis consectetur purus sit amet fermentum. Donec sed odio dui.

> Morbi leo risus, porta ac consectetur ac, vestibulum at eros.
"#
            })
        );
    }

    #[test]
    fn json_fm() {
        let input = r#"
{
    "title": "Hello World",
    "items": [1, "two"],
    "outer": {
        "inner": {
            "name": "Justo +++ Parturient"
        }
    }
}
Cras mattis?"#;
        let parsed = parse_document(input);

        assert_eq!(
            parsed,
            Ok(MatterhornDocument {
                front_matter: json!({
                    "title": "Hello World",
                    "items": [1, "two"],
                    "outer": {
                        "inner": {
                            "name": "Justo +++ Parturient"
                        }
                    }
                }),
                content: "Cras mattis?"
            })
        );
    }

    #[test]
    fn ordered_fm() {
        let input = r#"
---
title: Number one
aaaaa: Number two
zzzzz: Number three
bbbbb: Number four
b: Number five
---"#;
        let parsed = parse_document(input);

        assert_eq!(
            parsed,
            Ok(MatterhornDocument {
                front_matter: json!({
                    "title": "Number one",
                    "aaaaa": "Number two",
                    "zzzzz": "Number three",
                    "bbbbb": "Number four",
                    "b": "Number five",
                }),
                content: ""
            })
        );

        let doc = parsed.unwrap();
        let keys = doc
            .front_matter
            .as_object()
            .unwrap()
            .keys()
            .map(|s| &s[..])
            .collect::<Vec<_>>();

        assert_eq!(keys, vec!["title", "aaaaa", "zzzzz", "bbbbb", "b"]);
    }

    #[test]
    fn duplicate_keys_fm() {
        let input = r#"
---
title: Number one
title: Number two
title: Number three
title: Number four
title: Number five
---"#;
        let parsed = parse_document(input);

        assert_eq!(
            parsed,
            Ok(MatterhornDocument {
                front_matter: json!({ "title": "Number five" }),
                content: ""
            })
        );
    }

    #[test]
    fn unquoted_ellipsis_fm() {
        let input = r#"
---
comment: hello ... world
---"#;
        let parsed = parse_document(input);

        assert_eq!(
            parsed,
            Ok(MatterhornDocument {
                front_matter: json!({ "comment": "hello ... world" }),
                content: ""
            })
        );
    }
}
