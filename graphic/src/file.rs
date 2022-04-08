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

pub type LineSeries = BTreeMap<String, crate::LineSeries>;

use std::collections::hash_map::Entry;
use std::collections::BTreeMap;
use std::path::Path;
use futures::StreamExt;

use log_new::value::*;
use log_new::files::csv::*;
use log_new::convert::{stream::*, iterator::*};

pub(crate) enum Engine {
    Top,
    Low,
    TopLow,
}

pub(crate) async fn open_csv(file_path: impl AsRef<Path>, eng: Engine, names: &[&str]) -> Option<LineSeries> {
    let values = read_values(file_path.as_ref().with_extension("csv"))?;
    let values = fullvalue_to_elk(values);
    let lines = values_to_line(futures::stream::iter(values));
    let lines = match eng {
        Engine::Top => log_new::stat_info::simple::filter_half_top(lines).boxed(),
        Engine::Low => log_new::stat_info::simple::filter_half_low(lines).boxed(),
        _ => log_new::stat_info::simple::filter_half_top(lines).boxed(),
    };

    let series = lines2series(lines, names).await;

    Some(series)
}

pub fn open_top_low(file_path: impl AsRef<Path>, names: &[&str]) -> Option<LineSeries> {
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
