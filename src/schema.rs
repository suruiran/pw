use std::collections::HashMap;

use ratatui::{layout::Rect, widgets::Paragraph};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ArgvKind {
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
pub struct Argv {
    pub(crate) name: String,
    #[serde(flatten)]
    pub(crate) kind: ArgvKind,
    pub(crate) required: Option<bool>,
    pub(crate) repeatable: Option<bool>,

    #[serde(alias = "desc")]
    pub(crate) description: Option<String>,

    pub(crate) conflicts_with: Option<Vec<String>>,
    pub(crate) depends_on: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Command {
    pub(crate) exe: String,
    pub(crate) description: Option<String>,
    pub(crate) args: Option<Vec<Argv>>,
    pub(crate) subs: Option<HashMap<String, Command>>,
}

impl Command {
    pub fn has_args(&self) -> bool {
        return self.args.as_ref().is_some_and(|v| !v.is_empty());
    }

    pub fn has_subs(&self) -> bool {
        return self.subs.as_ref().is_some_and(|v| !v.is_empty());
    }

    pub fn is_empty(&self) -> bool {
        return !self.has_args() && !self.has_subs();
    }
}
