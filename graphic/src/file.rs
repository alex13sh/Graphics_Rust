pub fn save_svg(svg_text: &str, date_time: crate::DateTime) {
    use std::io::Write;
    let svg_name = format!("plot_{}", log_new::date_time_to_string_name_short(&date_time));
    let mut f = std::fs::File::create(format!("./{}.svg", svg_name)).unwrap();
    f.write(svg_text.as_bytes());
    f.flush();
    
        use std::process::Command;
        let mut cmd = Command::new("inkscape");
            cmd.arg("-z").arg("-d 320")
            .arg(format!("./{}.svg", svg_name))
            .arg("-e").arg(format!("./{}.png", svg_name));
        dbg!(&cmd);
            cmd.spawn().unwrap();
}