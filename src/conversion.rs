use saphyr::{Scalar, Yaml};

pub fn yaml_to_json(yaml: Yaml) -> Result<serde_json::Value, String> {
    match yaml {
        Yaml::Value(Scalar::FloatingPoint(x)) => Ok(serde_json::Value::from(x.into_inner())),
        Yaml::Value(Scalar::Integer(x)) => Ok(serde_json::Value::from(x)),
        Yaml::Value(Scalar::String(x)) => Ok(serde_json::Value::from(x)),
        Yaml::Value(Scalar::Boolean(x)) => Ok(serde_json::Value::from(x)),
        Yaml::Value(Scalar::Null) => Ok(serde_json::Value::Null),
        Yaml::Sequence(mut arr) => Ok(serde_json::Value::Array(
            arr.drain(..).map(yaml_to_json).collect::<Result<_, _>>()?,
        )),
        Yaml::Mapping(mut hash) => Ok(serde_json::Value::Object(
            hash.drain()
                .map(|(key, val)| {
                    let key = match key {
                        Yaml::Value(Scalar::Integer(x)) => Ok(x.to_string()),
                        Yaml::Value(Scalar::String(x)) => Ok(x.to_string()),
                        Yaml::Value(Scalar::FloatingPoint(x)) => Ok(x.to_string()),
                        key => Err(format!("Bad key {key:?}")),
                    }?;
                    let val = yaml_to_json(val)?;
                    Ok((key, val))
                })
                .collect::<Result<_, String>>()?,
        )),
        Yaml::BadValue => Err(format!("Bad value encountered when converting YAML")),
        Yaml::Alias(_) => Err(format!("Unresolved alias encountered when converting YAML")),
        Yaml::Representation(x, _, _) => Err(format!(
            "Encountered unexpected representation when converting YAML: {x:?}"
        )),
    }
}
