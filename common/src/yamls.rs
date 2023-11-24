use std::collections::HashMap;

use serde::ser::Serialize;
use serde_yaml::Result;

pub fn make_default_template<T: ?Sized + Serialize + Default>(doc: HashMap<String, (String, Option<String>)>) -> Result<String> {
    make_template(&T::default(), doc)
}

/// Turn this struct into a template by replacing all optional values with a commented-out
/// version of the field and no value. This assumes all fields that are `None` in are optional
///
/// `some_field: null` becomes `# some_field:`
pub fn make_template<T: ?Sized + Serialize>(thing: &T, doc: HashMap<String, (String, Option<String>)>) -> Result<String> {
    Ok(string_to_template(serde_yaml::to_string(thing)?, doc))
}

pub fn string_to_template(string: String, doc: HashMap<String, (String, Option<String>)>) -> String {
    string
        .split_inclusive('\n')
        .map(|line| match strip_default(line) {
            Some(line) => match doc.get(line.trim()) {
                Some((desc, Some(default))) => format!("# {desc} \n# {line}: {default}\n"),
                Some((desc, None)) => format!("# {desc} \n# {line}:\n"),
                _ => format!("# {line}:\n"),
            },
            None => line.to_owned(),
        })
        .collect()
}

fn strip_default(line: &str) -> Option<&str> {
    [": null\n", ": false\n", ": ''\n", ": []\n", ": {}\n"].iter().find_map(|suffix| line.strip_suffix(suffix))
}
