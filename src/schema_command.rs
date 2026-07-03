use crate::{
    model_state::{CmdWithValue, ModelState, get_cmdv},
    schema::{self, Argument},
};

pub(crate) enum AvailableArgs<'a> {
    None,
    Unchanged(Vec<&'a Argument>),
    Changed(Vec<Argument>),
}

impl<'a> AvailableArgs<'a> {
    pub(crate) fn len(&self) -> usize {
        match self {
            AvailableArgs::None => 0,
            AvailableArgs::Unchanged(arguments) => arguments.len(),
            AvailableArgs::Changed(arguments) => arguments.len(),
        }
    }

    pub(crate) fn iter(&self) -> Box<dyn Iterator<Item = &Argument> + '_> {
        match self {
            AvailableArgs::None => Box::new(std::iter::empty()),
            AvailableArgs::Unchanged(arguments) => Box::new(arguments.iter().copied()),
            AvailableArgs::Changed(arguments) => Box::new(arguments.iter()),
        }
    }
}

impl schema::Command {
    pub(crate) fn available_args<'a>(
        &'a self,
        ms: &ModelState,
        path: &[String],
    ) -> AvailableArgs<'a> {
        if let Some(cmdv) = get_cmdv(ms, path) {
            // todo check conflicts_with/depends_on
            return AvailableArgs::Changed(vec![]);
        }
        match self.args.as_ref() {
            Some(args) => {
                return AvailableArgs::Unchanged(args.iter().collect());
            }
            None => {
                return AvailableArgs::None;
            }
        }
    }
}
