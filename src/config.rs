use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub(crate) enum ArgvKind {
    Select {
        options: Option<Vec<String>>,
        allow_custom: Option<bool>,
    },
    FilePath {
        globs: Option<Vec<String>>,
        dir_only: Option<bool>,
    },
    Str {
        from_file: Option<bool>,
        regexp: Option<String>,
        secret: Option<bool>,
        textarea: Option<bool>,
    },
    Number {
        min: Option<f64>,
        max: Option<f64>,
        int: Option<bool>,
    },
    Flag {},
    Pairs {
        key: Option<Vec<String>>,
        allow_custom_key: Option<bool>,
        value: Option<Box<ArgvKind>>,
        sep: Option<String>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Argv {
    name: String,
    #[serde(flatten)]
    kind: ArgvKind,
    required: Option<bool>,
    repeatable: Option<bool>,

    #[serde(alias = "desc")]
    description: Option<String>,

    conflicts_with: Option<Vec<String>>,
    depends_on: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Command {
    exe: String,
    description: Option<String>,
    args: Option<Vec<Argv>>,
    subs: Option<HashMap<String, Command>>,
}
