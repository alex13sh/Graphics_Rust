pub fn save_svg(svg_text: &str, date_time: crate::DateTime) {

    let dir = log_new::get_file_path("log/plot");

    use std::io::Write;
    let svg_name = format!("plot_{}", log_new::date_time_to_string_name_short(&date_time));
//     let file_name_csv = format!("{}/{}.svg", dir, svg_name);
    let file_name_svg = dir.join(&svg_name).with_extension("svg");
    dbg!(&file_name_svg);
    let mut f = std::fs::File::create(&file_name_svg).unwrap();
    f.write(svg_text.as_bytes());
    f.flush();
    
        use std::process::Command;
        let mut cmd = Command::new("inkscape");
            cmd.arg("-z").arg("-d 320")
            .arg(&file_name_svg)
            .arg("-e").arg(dir.join(&svg_name).with_extension("png"));
        dbg!(&cmd);
            cmd.spawn().unwrap();
}