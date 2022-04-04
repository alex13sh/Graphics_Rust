use plotters::prelude::*;
use std::collections::BTreeMap;

type LineSeries = BTreeMap<String, Vec<f32>>;

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
        let lines = values_simple_line_to_hashmap_f32(lines).take(5); 
        
        let mut series = LineSeries::new();
        for name in names {
            series.entry(name.to_string()).or_default();
        }
        // let lines = std::pin::Pin::new(&mut lines);
        let mut lines = lines.boxed();
        while let Some(line) = lines.next().await {
            for (name, value) in line.values {
                // series.entry(name).or_default().push(value);
                if let std::collections::btree_map::Entry::Occupied(ref mut ent) = series.entry(name) {
                    ent.get_mut().push(value);
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
            &["Виброскорость", "Выходной ток (A)", "Скорость двигателя"]));
        dbg!(series);
    }
}