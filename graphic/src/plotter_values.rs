use plotters::prelude::*;
use std::collections::BTreeMap;
use core::ops::Range;

type LineSeries = BTreeMap<String, crate::LineSeries>;

fn draw_series<'a, B, BE>(back: B, seconds_range: Range<f32>, series: impl Iterator<Item=&'a crate::LineSeries>) 
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
    let mut chart = cc_build(&root_area, "Graphic", 0.0..100.0);

    for (s, c) in series.filter(|s| s.points.len() >=2 ).zip(0..) {
        let points = &s.points;
        let date_start = points.first().unwrap().date_time.clone();
        let ls = plotters::prelude::LineSeries::new(
            points.iter().map(|p| (dlt_time_f32(&date_start, p.date_time), p.value)),
                &Palette99::pick(c),
            );
        chart.draw_series(ls).unwrap();
    }
}

mod tests {
    use std::collections::hash_map::Entry;

    use super::LineSeries;

    use futures::StreamExt;
    use log_new::value::*;
    use log_new::files::csv::*;
    use log_new::convert::{stream::*, iterator::*};

    async fn open_csv(file_path: &str, names: &[&str]) -> Option<LineSeries> {
        let values = read_values(format!("{}.csv", file_path))?;
        let values = fullvalue_to_elk(values);
        let lines = values_to_line(futures::stream::iter(values));
        let lines = log_new::stat_info::simple::filter_half_low(lines);
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
        Some(series)
    }

    #[test]
    fn test_plot_half() {
        let dir = "/home/user/.local/share/graphicmodbus/log/values/csv_raw";
        let name = "2022_03_22-17_17_18";
        let file_path = format!("{}/{}", dir, name);

        let series = futures::executor::block_on(open_csv(&file_path, 
            &["Виброскорость", "Выходной ток (A)", "Скорость двигателя"])).unwrap();
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
        let back = super::SVGBackend::with_string(&mut svg_text, (1280, 720));
        super::draw_series(back, seconds_range, series.values());
        crate::file::save_svg(&svg_text, date_time_start);
    }
}