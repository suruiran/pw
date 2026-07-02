pub(crate) fn load_schema_content(
    cfg_dir: &String,
    cfg_base_name: &String,
) -> Option<(String, String)> {
    let mut cfg_content: Option<(String, String)> = None;
    for cfg_ext in vec![".toml", ".json"] {
        let cfg_path =
            std::path::PathBuf::from(cfg_dir).join(format!("{}{}", cfg_base_name, cfg_ext));

        match std::fs::read_to_string(cfg_path) {
            Ok(c) => {
                cfg_content = Some((cfg_ext.to_string(), c));
                break;
            }
            _ => {}
        }
    }
    return cfg_content;
}
