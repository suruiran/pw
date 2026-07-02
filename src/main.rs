use clap::Parser;

mod entry;
mod load_schema;
mod repl;
mod schema;
mod ui;
mod ui_argv;
mod ui_content;
mod ui_eleinfo;
mod ui_event;
mod ui_style;
mod utils;

fn main() -> Result<(), String> {
    let entry = entry::Entry::parse();

    #[cfg(debug_assertions)]
    {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(entry.log.unwrap_or("./debug.log".to_string()))
            .expect("failed to create log file");
        tracing_subscriber::fmt().with_writer(file).init();
    }

    let cmdpath = utils::find_executable(&entry.cmd);
    if cmdpath.is_none() {
        return Err(format!("`{}` is not found or not a executable", entry.cmd));
    }
    let cfg_dirs = entry.config.unwrap_or_else(|| {
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

    let cfg_base_name = entry.using.unwrap_or_else(|| {
        let msg = format!(
            "failed to get base filename from command path: {}",
            &entry.cmd
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
        return Err(format!("can not read schema for command `{}`", entry.cmd));
    }

    let (filekind, filecontent) = cfg_content.unwrap();

    let schema: schema::Command;
    match filekind.as_str() {
        ".json" => {
            let dr: Result<schema::Command, serde_json::Error> = serde_json::from_str(&filecontent);
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
            let dr: Result<schema::Command, toml::de::Error> = toml::from_str(&filecontent);
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

    match ui::ui(schema) {
        Ok(args) => {
            if args.len() < 1 {
                return Err("unreachable code: empty output args".to_string());
            }
            if entry.dryrun.is_some() && entry.dryrun.unwrap() {
                println!("{}", args.join(" "));
                return Ok(());
            }
            let mut cmd = std::process::Command::new(args[0].clone());
            cmd.args(&args[1..]);
            match cmd.status() {
                Ok(s) => {
                    if s.success() {
                        return Ok(());
                    }
                    return Err(format!("{:?}", s));
                }
                Err(e) => {
                    return Err(e.to_string());
                }
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}
