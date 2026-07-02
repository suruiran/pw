use clap::Parser;

mod config;
mod entry;
mod load_schema;
mod utils;

fn main() -> Result<(), String> {
    let args = entry::Entry::parse();

    let cmdpath = utils::find_executable(&args.cmd);
    if cmdpath.is_none() {
        return Err(format!("`{}` is not found or not a executable", args.cmd));
    }
    let cfg_dirs = args.config.unwrap_or_else(|| {
        return vec![
            std::env::var("PW_CURRENT_USER_CUSTOM_CONFIGS").unwrap_or("".to_string()),
            dirs::config_dir()
                .expect("read user config dir failed")
                .join("pw")
                .to_string_lossy()
                .to_string(),
            std::env::var("PW_HOST_CUSTOM_CONFIGS").unwrap_or("".to_string()),
        ];
    });

    let cfg_base_name = args.using.unwrap_or_else(|| {
        let msg = format!(
            "failed to get base filename from command path: {}",
            &args.cmd
        );
        let _cmdpath = cmdpath.as_ref().expect(&msg);
        let stem = if cfg!(windows) {
            _cmdpath.file_stem()
        } else {
            _cmdpath.file_name()
        };
        return stem
            .expect(&msg)
            .to_string_lossy()
            .to_string()
            .to_lowercase();
    });

    let mut cfg_content: Option<(String, String)> = None;
    for cfg_dir in cfg_dirs.iter() {
        if cfg_dir == "" {
            continue;
        }
        cfg_content = load_schema::load_schema_content(cfg_dir, &cfg_base_name);
        if cfg_content.is_some() {
            break;
        }
    }
    if cfg_content.is_none() {
        return Err(format!("can not read schema for command `{}`", args.cmd));
    }

    let (filekind, filecontent) = cfg_content.unwrap();

    let schema: config::Command;
    match filekind.as_str() {
        ".json" => {
            let dr: Result<config::Command, serde_json::Error> = serde_json::from_str(&filecontent);
            match dr {
                Ok(v) => {
                    schema = v;
                }
                Err(e) => {
                    return Err(e.to_string());
                }
            }
        }
        ".toml" => {
            let dr: Result<config::Command, toml::de::Error> = toml::from_str(&filecontent);
            match dr {
                Ok(v) => {
                    schema = v;
                }
                Err(e) => {
                    return Err(e.to_string());
                }
            }
        }
        _ => {
            return Err(format!(
                "unreachable code: unexpected config file ext: {}",
                &filekind
            ));
        }
    }

    println!("{:?}", schema);

    return Ok(());
}
