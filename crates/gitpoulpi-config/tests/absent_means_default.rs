//! A key (or a whole table) missing from `config.toml` means the schema
//! default, never `bool::default()` / zero. Walks every table of the default
//! config: empties it in an otherwise default document, parses, and expects
//! the default back. Catches a `#[serde(default)]` on a field of a struct
//! whose `Default` says otherwise (a default-true bool read as false).

use gitpoulpi_config::Config;

fn table_paths(value: &toml::Value, at: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
    if let toml::Value::Table(table) = value {
        for (key, child) in table {
            if child.is_table() {
                at.push(key.clone());
                out.push(at.clone());
                table_paths(child, at, out);
                at.pop();
            }
        }
    }
}

fn emptied(doc: &toml::Value, path: &[String]) -> toml::Value {
    let mut doc = doc.clone();
    let mut cursor = &mut doc;
    for (i, key) in path.iter().enumerate() {
        let table = cursor.as_table_mut().unwrap();
        if i + 1 == path.len() {
            table.insert(key.clone(), toml::Value::Table(Default::default()));
            break;
        }
        cursor = table.get_mut(key).unwrap();
    }
    doc
}

#[test]
fn an_absent_key_or_table_means_the_schema_default() {
    let default = toml::Value::try_from(Config::default()).unwrap();
    let (none, _) = Config::from_toml_str("").unwrap();
    assert_eq!(toml::Value::try_from(none).unwrap(), default, "empty file");

    let mut paths = Vec::new();
    table_paths(&default, &mut Vec::new(), &mut paths);
    assert!(paths.iter().any(|p| p == &["general".to_string()]));
    for path in paths {
        let doc = emptied(&default, &path);
        let (config, _) = Config::from_toml_str(&toml::to_string(&doc).unwrap()).unwrap();
        let back = toml::Value::try_from(config).unwrap();
        assert_eq!(
            back,
            default,
            "an empty [{}] must read as the defaults",
            path.join(".")
        );
    }

    // And the general table one key at a time.
    let general = default["general"].as_table().unwrap().clone();
    for key in general.keys() {
        let mut doc = default.clone();
        doc["general"].as_table_mut().unwrap().remove(key);
        let (config, _) = Config::from_toml_str(&toml::to_string(&doc).unwrap()).unwrap();
        assert_eq!(
            toml::Value::try_from(config).unwrap(),
            default,
            "general.{key} absent must read as the default"
        );
    }
}
