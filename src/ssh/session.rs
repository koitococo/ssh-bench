const MIB: u64 = 1024 * 1024;

pub fn render_throughput_command(
    template: &str,
    file: &str,
    size_bytes: u64,
) -> Result<String, String> {
    if !template.contains("{file}") || !template.contains("{count}") {
        return Err("template must contain {file} and {count}".to_string());
    }

    let count = size_bytes.div_ceil(MIB);
    Ok(template
        .replace("{file}", file)
        .replace("{count}", &count.to_string()))
}
