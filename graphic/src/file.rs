pub fn save_svg(&self) {
    if let Some(svg_text) = self.make_svg(self.dt_start, self.view_port.end, true) {
        use std::io::Write;
        let svg_name = format!("plot_{}", log::date_time_to_string_name(&self.dt_start.into()));
        let mut f = std::fs::File::create(format!("./plot/{}.svg", svg_name)).unwrap();
        f.write(svg_text.as_bytes());
        f.flush();
        
        use std::process::Command;
        let _ = Command::new("inkscape")
            .arg("-z").arg("-d 320")
            .arg(format!("./plot/{}.svg", svg_name))
            .arg("-e").arg(format!("./plot/{}.png", svg_name))
            .spawn().unwrap();
    }
}