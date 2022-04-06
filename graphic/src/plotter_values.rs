use plotters::prelude::*;
use std::collections::BTreeMap;
use core::ops::Range;

type LineSeries = BTreeMap<String, crate::LineSeries>;

fn draw_series<'a, B, BE>(back: B, name: &str, seconds_range: Range<f32>, series: impl Iterator<Item=&'a crate::LineSeries>) 
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

async fn lines2series(
        lines:  impl futures::Stream<Item=log_new::value::SimpleValuesLine> + Send,
        names: &[&str]
    ) -> LineSeries {

    use futures::StreamExt;
    use log_new::convert::{stream::*, iterator::*};

    let lines = values_simple_line_to_hashmap_f32(lines);
    let mut series = LineSeries::new();
    for name in names {
        series.entry(name.to_string()).or_insert(crate::LineSeries::new(name));
    }
    // let lines = std::pin::Pin::new(&mut lines);
    let mut lines = lines.boxed();
    while let Some(line) = lines.next().await {
        for (name, value) in line.values {
            // series.entry(name).or_default().push(value);
            if let std::collections::btree_map::Entry::Occupied(ref mut ent) = series.entry(name) {
                ent.get_mut().addPoint(crate::DatePoint {
                    date_time: line.date_time,
                    value
                });
            }
        }
    }
    series
}

mod tests {
    use std::collections::hash_map::Entry;
    use std::collections::BTreeMap;

    use super::LineSeries;

    use futures::StreamExt;
    use log_new::value::*;
    use log_new::files::csv::*;
    use log_new::convert::{stream::*, iterator::*};

    enum Engine {
        Top,
        Low,
        TopLow,
    }

    async fn open_csv(file_path: &str, eng: Engine, names: &[&str]) -> Option<LineSeries> {
        let values = read_values(format!("{}.csv", file_path))?;
        let values = fullvalue_to_elk(values);
        let lines = values_to_line(futures::stream::iter(values));
        let lines = match eng {
            Engine::Top => log_new::stat_info::simple::filter_half_top(lines).boxed(),
            Engine::Low => log_new::stat_info::simple::filter_half_low(lines).boxed(),
            _ => log_new::stat_info::simple::filter_half_top(lines).boxed(),
        };

        let series = super::lines2series(lines, names).await;

        Some(series)
    }

    fn open_top_low(file_path: &str, names: &[&str]) -> Option<LineSeries> {
        let mut series = futures::executor::block_on(
            open_csv(&file_path, Engine::Low, names))?;
        series.get_mut("Скорость двигателя").unwrap().set_graphic_second(true);
        series.get_mut("Индикация текущей выходной мощности (P)").unwrap().convert_to_i32();

        let mut series_full: LineSeries = series.into_iter()
        .map(|(name, mut s)|{
            let name = name + " Низ.";
            s.name = name.clone();
            (name, s)
        }).collect();

        let mut series = futures::executor::block_on(
            open_csv(&file_path, Engine::Top, names))?;
        series.get_mut("Скорость двигателя").unwrap().set_graphic_second(true);
        series.get_mut("Индикация текущей выходной мощности (P)").unwrap().convert_to_i32();

        let mut series: LineSeries = series.into_iter()
        .map(|(name, mut s)|{
            let name = name + " Верх.";
            s.name = name.clone();
            (name, s)
        }).collect();
        series_full.append(&mut series);
        Some(series_full)
    }

    #[test]
    fn test_plot_half() {
        let dir = "/home/user/.local/share/graphicmodbus/log/values/csv_raw";
        let name = "2022_03_29-13_58_12";
        let file_path = format!("{}/{}", dir, name);

        let series = open_top_low(&file_path, 
            &["Виброскорость", "Скорость двигателя",
            "Выходной ток (A)", "Индикация текущей выходной мощности (P)"]).unwrap();
        // dbg!(&series);

        let mut date_time_start; //= seconds_range.first().unwrap().date_time.clone();
        let seconds_range = {
            let seconds_range = series.first_key_value().as_ref().unwrap().1.get_points();
            date_time_start = seconds_range.first().unwrap().date_time.clone();
            // seconds_range.first().unwrap().date_time..seconds_range.last().unwrap().date_time
            let last_time = seconds_range.last().unwrap().date_time.timestamp_millis() - seconds_range.first().unwrap().date_time.timestamp_millis();
            let last_time = (last_time as f32 / 100.0).round() / 10.0;
            0.0..last_time
        };
        let mut svg_text = String::new();
        let back = super::SVGBackend::with_string(&mut svg_text, (1920, 900));
        super::draw_series(back, name, seconds_range, series.values());
        crate::file::save_svg(&svg_text, date_time_start);
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
