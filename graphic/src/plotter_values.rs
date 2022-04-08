use plotters::prelude::*;
use std::collections::BTreeMap;
use core::ops::Range;

pub(crate) fn draw_series<'a, B, BE>(back: B, name: &str, seconds_range: Range<f32>, series: impl Iterator<Item=&'a crate::LineSeries>)
where
    BE: std::error::Error + Send + Sync,
    B: plotters::prelude::DrawingBackend<ErrorType=BE>,
{
    use std::ops::Sub;

    let root_area = back.into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let dlt_time_f32 = |dt_start: &crate::DateTime, dt: crate::DateTime|
            if let Ok(std) = dt.sub(dt_start.clone()).to_std() {
                std.as_secs_f32()
            } else {
                0_f32
            };

    let cc_build = |on, graphic_name: &str, range_Y: Range<f32>| {
        ChartBuilder::on(on)
        .x_label_area_size(25_i32)
        .y_label_area_size(40_i32)
        .right_y_label_area_size(40_i32)
        .margin(5_i32)
//             .margin_right(20)
        .caption(
            graphic_name, // date name
            ("sans-serif", 20).into_font(),
        ).build_ranged( // build_cartesian_2d
            seconds_range.clone(),
            range_Y
        ).unwrap()
        };
    let mut cc_speed = {
        let mut cc = cc_build(&root_area, name, //"Скорость",
        0_f32..60_f32)
        .set_secondary_coord(seconds_range.clone(),
        0_f32..20_000_f32);
        cc.configure_mesh()
            .x_labels(20).y_labels(8)
            .y_desc("")
            .y_label_formatter(&|x| format!("{:2.}", x))
            .draw().unwrap();
        cc.configure_secondary_axes()
            .x_labels(20).y_labels(20)
            .y_desc("Скорость (об./м)")
            .y_label_formatter(&|x| format!("{}", *x as u32))
            .draw().unwrap();
            cc};

    // let mut chart = cc_build(&root_area, "Graphic", 0.0..100.0);

    for (s, c) in series.filter(|s| s.points.len() >=2 ).zip(0..) {
        let points = &s.points;
        let date_start = points.first().unwrap().date_time.clone();
        let ls = plotters::prelude::LineSeries::new(
            points.iter().map(|p| (dlt_time_f32(&date_start, p.date_time), p.value)),
            &Palette99::pick(c),
            );
        if s.is_graphic_second() {
            cc_speed.draw_secondary_series(ls).unwrap()
        } else {
            cc_speed.draw_series(ls).unwrap()
        }
            .label(&s.name)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &Palette99::pick(c)));;
    }
    cc_speed.configure_series_labels()
    .background_style(&WHITE.mix(0.8))
    .border_style(&BLACK)
    .draw().unwrap();
}



mod tests {

    #[test]
    fn test_plot_half() {
//         let dir = "/home/user/.local/share/graphicmodbus/log/values/csv_raw";
        let name = "2022_03_29-13_58_12";
        crate::file::csv2svg(name);
    }
    
    #[test]
    fn test_inskape() {
        use std::process::Command;
        let mut cmd = Command::new("/usr/bin/inkscape");
            // cmd.arg("-z").arg("-d 320")
            // .arg(format!("/home/user/projects/rust/SimpleUI_Rust/graphic/{}.svg", svg_name))
            // .arg("-e").arg(format!("/home/user/projects/rust/SimpleUI_Rust/graphic/{}.png", svg_name));
            cmd.arg("--version");
        dbg!(&cmd);
        let cmd = cmd.spawn();
        dbg!(&cmd);
        cmd.unwrap();
    }
}
