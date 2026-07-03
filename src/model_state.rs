use crate::utils::FastMap;

#[derive(Debug, Default)]
pub(crate) struct Argv {
    pub(crate) name: String,
    pub(crate) value: Option<Vec<String>>,
}

#[derive(Debug, Default)]
pub(crate) struct CmdWithValue {
    pub(crate) name: String,
    pub(crate) args: FastMap<String, Argv>,
    pub(crate) current: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct ModelState {
    pub(crate) stack: Vec<CmdWithValue>,
    pub(crate) cache: FastMap<String, CmdWithValue>,
    pub(crate) current: Option<usize>,

    pub(crate) inputid: String,
    pub(crate) inputtemp: String,
}

pub(crate) fn get_cmdv<'a>(ms: &'a ModelState, path: &[String]) -> Option<&'a CmdWithValue> {
    if ms.stack.len() >= path.len() {
        let mut current_ref: Option<&'a CmdWithValue> = None;
        let mut matches = true;
        for (cmdv, name) in ms.stack.iter().zip(path.iter()) {
            if &cmdv.name != name {
                matches = false;
                break;
            }
            current_ref = Some(cmdv);
        }
        if matches {
            return current_ref;
        }
    }
    return ms.cache.get(path.join("/").as_str());
}

pub(crate) fn get_cmdv_mut<'a>(
    ms: &'a mut ModelState,
    path: &[String],
) -> Option<&'a mut CmdWithValue> {
    if ms.stack.len() >= path.len() {
        let mut current_ref: Option<&mut CmdWithValue> = None;
        let mut matches = true;
        for (cmdv, name) in ms.stack.iter_mut().zip(path.iter()) {
            if &cmdv.name != name {
                matches = false;
                break;
            }
            current_ref = Some(cmdv);
        }
        if matches {
            return current_ref;
        }
    }
    return ms.cache.get_mut(path.join("/").as_str());
}

pub(crate) fn get_argv<'a>(
    ms: &'a ModelState,
    path: &[String],
    argn: &str,
) -> Option<&'a Vec<String>> {
    return get_cmdv(ms, path)?.args.get(argn)?.value.as_ref();
}

pub(crate) fn get_argv_mut<'a>(
    ms: &'a mut ModelState,
    path: &[String],
    argn: &str,
) -> Option<&'a mut Vec<String>> {
    return get_cmdv_mut(ms, path)?.args.get_mut(argn)?.value.as_mut();
}
