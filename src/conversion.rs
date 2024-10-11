use saphyr::Yaml;

pub fn yaml_to_json(yaml: Yaml) -> Result<serde_json::Value, String> {
    match yaml {
        Yaml::Real(x) => {
            if let Ok(x) = x.parse::<serde_json::Number>() {
                Ok(serde_json::Value::Number(x))
            } else {
                // Yaml supports some numbers that can't be represented in JSON e.g. .nan and .inf
                // So we return a null if we encounter one of these.
                Ok(serde_json::Value::Null)
            }
        }
        Yaml::Integer(x) => Ok(serde_json::Value::from(x)),
        Yaml::String(x) => Ok(serde_json::Value::from(x)),
        Yaml::Boolean(x) => Ok(serde_json::Value::from(x)),
        Yaml::Null => Ok(serde_json::Value::Null),
        Yaml::Array(mut arr) => Ok(serde_json::Value::Array(
            arr.drain(..).map(yaml_to_json).collect::<Result<_, _>>()?,
        )),
        Yaml::Hash(mut hash) => Ok(serde_json::Value::Object(
            hash.drain()
                .map(|(key, val)| {
                    let key = match key {
                        Yaml::Integer(x) => Ok(x.to_string()),
                        Yaml::String(x) => Ok(x),
                        Yaml::Real(x) => Ok(x),
                        key => Err(format!("Bad key {key:?}")),
                    }?;
                    let val = yaml_to_json(val)?;
                    Ok((key, val))
                })
                .collect::<Result<_, String>>()?,
        )),
        Yaml::BadValue => Err(format!("Bad value encountered when converting YAML")),
        Yaml::Alias(_) => Err(format!("Unresolved alias encountered when converting YAML")), // TODO: Think about how to handle this
    }
}
