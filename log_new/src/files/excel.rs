mod sheet;
use sheet::SheetInner;

use umya_spreadsheet::structs::*;
//     use excel::*;
use super::inner::*;
use crate::LogState;
use crate::value::simple;

use coarse_prof::profile;

pub struct File {
    book: Spreadsheet,
    file_path: PathBuf,
}

impl File {
    pub fn create(file_path: impl AsRef<Path>) -> Self {
        Self {
            book: Self::new_file(),
            file_path: file_path.as_ref().into(),
        }
    }
    fn new_file() -> Spreadsheet {

        // let mut spreadsheet = Spreadsheet::default();
        // // spreadsheet.set_theme(Theme::get_defalut_value());
        // // spreadsheet.set_stylesheet_defalut_value();
        // spreadsheet
        let mut sht = umya_spreadsheet::new_file();
        let pos = sht.get_sheet_collection().iter().position(|ws| ws.get_title() == "Sheet1");
        if let Some(index) = pos {
            sht.get_sheet_collection_mut().remove(index);
        }
        sht
    }
    pub fn save(&self) {
        umya_spreadsheet::writer::xlsx::write(&self.book, &self.file_path).unwrap();
    }

    pub fn open_sheet(&mut self, name: &'static str) -> Sheet<&mut Worksheet> 
    {
        let sht = if self.book.get_sheet_by_name_mut(name).is_ok() {
            self.book.get_sheet_by_name_mut(name).unwrap()
        } else {
            self.book.new_sheet(name).unwrap()
        };
        Sheet::from(
            sht
        )
    }
    pub fn set_sheet(&mut self, mut ws: Worksheet, name: &'static str) {
        ws.set_title(name);
        self.book.add_sheet(ws).unwrap();
    }
}


// impl Drop for File {
//     fn drop(&mut self) {
//         coarse_prof::write(&mut std::io::stdout()).unwrap();
//     }
// }

pub struct Sheet<Sh: SheetInner> {
//         file: &'f mut File,
//         name: &'static str,
    ws: Sh,
    rows: u32,
}

impl <Sh> From<Sh> for Sheet<Sh> 
    where Sh: SheetInner
{
    fn from(v: Sh) -> Self {
        Self {
            ws: v,
            rows: 0,
        }
    }
}

impl Sheet <Worksheet> {
    pub fn new() -> Self {
        let ws = Worksheet::default();
        // ws.set_title(name.into());
        Self {
            ws,
            rows: 0,
        }
    }
}

impl <Sh> Sheet <Sh> 
    where Sh: SheetInner
{
    pub async fn write_values(&mut self, values: impl Stream<Item=simple::ValuesMap> + std::marker::Unpin) {
        let pos = (1, 1);
        let mut values = values.enumerate().peekable();
        
        let l = if let Some(ref l) = std::pin::Pin::new(&mut values).peek().await {&l.1}
        else {return};

        self.ws.get_cell_by_column_and_row_mut(pos.0 + 0, pos.1 + 0)
                .set_value("Время");
        let dt_start = l.date_time.clone();
        
        for (col, name) in l.values.keys().enumerate() {
            self.ws.get_cell_by_column_and_row_mut(pos.0 + col+1, pos.1).set_value(name);
        }
        // self.rows += 1;
    
        // self.ws.calculation_auto_width();

        while let Some((row, l)) = values.next().await {
            let time = l.date_time.timestamp_millis() - dt_start.timestamp_millis();
            let time = (time as f32 / 100.0).round() / 10.0;
            self.ws.get_cell_by_column_and_row_mut(pos.0 + 0, pos.1 + row+1)
                .set_value(time.to_string());
            for (col, v) in l.values.values().enumerate() {
                self.ws.get_cell_by_column_and_row_mut(pos.0 + col+1, pos.1 + row+1).set_value(v);
            }
            self.rows += 1;
        };
    }
    pub fn write_state(&mut self, pos: (usize, usize), state: LogState) {
        let mut fields = Vec::new();
        fields.push(("Время запуска", state.date_time.unwrap().to_string()));
        fields.push(("Время работы (сек)", state.time_all.to_string()));
        fields.push(("Время разгона (сек)", state.time_acel.to_string()));
        fields.push(("Обороты двигателя (об/мин)", state.hz_max.to_string()));
        fields.push(("Максимальная вибрация", state.vibro_max.to_string()));
        fields.push(("Зона вибрации (об/мин)", state.hz_vibro.to_string()));
        fields.push(("Максимальный ток", state.tok_max.to_string()));
        fields.push(("Максимальная мощность", state.watt_max.to_string()));
        
        for (f, row) in fields.into_iter().zip((pos.1+1)..) {
            self.ws.get_cell_by_column_and_row_mut(1+pos.0, row).set_value(f.0);
            self.ws.get_cell_by_column_and_row_mut(2+pos.0, row).set_value(f.1);
        }
        for (f, row) in state.temps.into_iter().zip((pos.1+10)..) {
            self.ws.get_cell_by_column_and_row_mut(pos.0+1, row).set_value(f.0);
            self.ws.get_cell_by_column_and_row_mut(pos.0+2, row).set_value(format!("{:.2}", f.1.0));
            self.ws.get_cell_by_column_and_row_mut(pos.0+3, row).set_value(format!("{:.2}", f.1.1));
        }
    }

    pub fn draw_graphic(&mut self, pos: (&str, &str), columns: &[&str]) {
        let area_time = self.ws.make_coordinates_columns(&["Время"], self.rows).swap_remove(0).1;

        let chart_series_list = self.ws.make_coordinates_columns(columns, self.rows);
        let area_chart_series_list = chart_series_list.iter()
            .map(|&(_, ref area)| area.as_str()).collect();
        let name_series_list = chart_series_list.iter()
            .map(|&(ref name, _)| name.as_str()).collect();
        dbg!(&area_chart_series_list);
        // vec![
        //     "Sheet1!$A$1:$A$10",
        //     "Sheet1!$B$1:$B$10",
        // ];
        self.ws.new_chart_liner(pos.0, pos.1, &area_time, name_series_list, area_chart_series_list);
    }
}

use crate::value::SimpleValuesLine;
pub fn filter_half(vin: impl Stream<Item=SimpleValuesLine>) -> impl Stream<Item=SimpleValuesLine> {
    filter_lines_map(vin, |sensor_name| {
        let b = match sensor_name {
        "Виброскорость" | "Выходной ток (A)" | "Скорость двигателя" | "Индикация текущей выходной мощности (P)" | "Выходное напряжение (E)" => true,
        "Заданная частота (F)" | "Напряжение на шине DC" | "Наработка двигателя (дни)" | "Наработка двигателя (мин)" => false,
        sensor_name if sensor_name.starts_with("Температура") => true,
        "Разрежение воздуха в системе" => true,
        _ => false,
    };
    if b {
        Some(sensor_name.to_owned())
    } else {
        None
    }
})

}

fn filter_lines_map(vin: impl Stream<Item=SimpleValuesLine>, f: fn(&str) -> Option<String>) -> impl Stream<Item=SimpleValuesLine> {
    use futures::StreamExt;
    vin.map(move |line| {
        SimpleValuesLine {
            date_time: line.date_time,
            values: line.values.into_vec().into_iter().filter_map(|mut v| {
                v.sensor_name = f(v.sensor_name.as_str())?; Some(v)
            } ).collect::<Vec<_>>().into_boxed_slice(),
        }
    })
}

fn join_lines_2(lines_1: impl Stream<Item=SimpleValuesLine>, lines_2: impl Stream<Item=SimpleValuesLine>) -> impl Stream<Item=SimpleValuesLine> {
    lines_1.zip(lines_2).map(|(l1, l2)| {
        SimpleValuesLine {
            date_time: l1.date_time,
            values: l1.values.into_vec().into_iter().chain(l2.values.into_vec().into_iter())
                .collect::<Vec<_>>().into_boxed_slice(),
        }
    })
}

use crate::value::ElkValuesLine;
pub fn write_file(file_path: impl AsRef<Path> + 'static, values_line: impl Stream<Item=SimpleValuesLine>) -> impl Future<Output=()> {
    use crate::async_channel::*;
    use crate::convert::{stream::*, iterator::*};
    use futures::future::join;

    async move {
        let file_path = file_path.as_ref();
        let mut f = File::create(file_path.with_extension("xlsx"));
        let s = f.open_sheet("Sheet1");
        let l1 = write_file_inner(values_line, s);
        l1.await;
        f.save();
    }
}

fn write_file_inner< Sh: SheetInner >(lines: impl Stream<Item=SimpleValuesLine> , mut sheet: Sheet<Sh>) -> impl Future<Output=()> {
    use crate::async_channel::*;
    use crate::convert::{stream::*, iterator::*};
    use futures::future::join;

    let (s, l1) = broadcast(500);

    let f_to_channel = lines.map(|l| Ok(l)).forward(s);
    let l2 = l1.clone();
    let f_from_channel = async move {
        let l1 = filter_half(l1);
        let l1 = values_simple_line_to_hashmap(l1);
//                 let l2 = crate::stat_info::simple::filter_half_low(l2);
//                 let l2 = values_line_to_simple(l2);

        let (_, stat) = join(
            sheet.write_values(l1),
            crate::stat_info::simple::calc(l2).fold(None, |_, s| async{Some(s)})
        ).await;
        dbg!("await");
        if let Some(stat) = stat {
            sheet.write_state((14,2), stat);
            sheet.draw_graphic(("M20", "AH60"), // (13, 20)
            &[
                "Виброскорость",
                "Выходной ток (A)", "Индикация текущей выходной мощности (P)",
                "Скорость двигателя",
            ]);
        }
    };
    async move {
        let _ = join(f_to_channel, f_from_channel).await;
    }
}

pub async fn write_file_2(file_path: impl AsRef<Path> + 'static, vl_top: impl Stream<Item=SimpleValuesLine>, vl_low: impl Stream<Item=SimpleValuesLine>)
{
    use crate::async_channel::*;
    use crate::convert::{stream::*, iterator::*};
    use futures::future::join;
    use futures::executor::block_on;

    let file_path = file_path.as_ref();
    let mut f = File::create(file_path.with_extension("xlsx"));
    // let sht_1 = Sheet::new();
    // write_file_inner(vl_top, sht_1).await;
    write_file_inner(vl_top, f.open_sheet("Верхний двигатель")).await;
    write_file_inner(vl_low, f.open_sheet("Нижний двигатель")).await;
    f.save();
}

pub async fn write_file_3(file_path: impl AsRef<Path> + 'static, lines: impl Stream<Item=ElkValuesLine>) {
    use std::time::Instant;

    use crate::async_channel::*;
    use crate::convert::{stream::*, iterator::*};
    use crate::stat_info::simple::*;
    use futures::join;

    let (s, lines_sink) = broadcast(4000);
    let f_to_channel_1 = lines.map(|l| Ok(l)).forward(s);
    let lines_top = filter_half_top(lines_sink.clone());
    let lines_low = filter_half_low(lines_sink);
    
    let (s, lines_sink) = broadcast(4000);
    let f_to_channel_2 = lines_top.map(|l| Ok(l)).forward(s);
    let lines_top_1 = lines_sink.clone();
    let lines_top_2 = lines_sink;

    let (s, lines_sink) = broadcast(4000);
    let f_to_channel_3 = lines_low.map(|l| Ok(l)).forward(s);
    let lines_low_1 = lines_sink.clone();
    let lines_low_2 = lines_sink;

    let f_list_top = async move {
        let mut ws = Worksheet::default();
        ws.set_title("Верхний двигатель");
        let sht = Sheet::from(&mut ws);
        // dbg!(Instant::now());
        write_file_inner(lines_top_1, sht).await;
        dbg!(Instant::now());
        ws
    };

    let f_list_low = async move {
        let mut ws = Worksheet::default();
        ws.set_title("Нижний двигатель");
        let sht = Sheet::from(&mut ws);
        // dbg!(Instant::now());
        write_file_inner(lines_low_1, sht).await;
        dbg!(Instant::now());
        ws
    };

    let f_list_first = async move {
        let mut ws = Worksheet::default();
        ws.set_title("Summary");
        let mut sht = Sheet::from(&mut ws);

        let lines_top_2 = filter_half(lines_top_2);
        let lines_top_2 = filter_lines_map(lines_top_2, |sensor_name| {
            if !sensor_name.starts_with("Температура") {
                Some(sensor_name.to_owned() + " (Верх.)")
            } else {
                None
            }
        });

        let lines_low_2 = filter_half(lines_low_2);
        let lines_low_2 = filter_lines_map(lines_low_2, |sensor_name| {
            if !sensor_name.starts_with("Температура") {
                Some(sensor_name.to_owned() + " (Ниж.)")
            } else {
                None
            }
        });

        let lines = join_lines_2(lines_top_2, lines_low_2);

        let lines = values_simple_line_to_hashmap(lines);
        // dbg!(Instant::now());
        // let lines = lines.take(1);

        sht.write_values(lines).await;
        sht.draw_graphic(("N3", "AJ41"), // (13, 20)
            &[
                "Виброскорость (Верх.)", "Виброскорость (Ниж.)",
                "Выходной ток (A) (Верх.)", "Выходной ток (A) (Ниж.)", 
                "Индикация текущей выходной мощности (P) (Верх.)", "Индикация текущей выходной мощности (P) (Ниж.)",
                "Скорость двигателя (Верх.)", "Скорость двигателя (Ниж.)",
            ]);

        dbg!(Instant::now());
        ws
    };

    let f = async move {
        dbg!(Instant::now());
        let (
            _, _, _,
            ws_top, ws_low,
            ws_1,
        )  = join!(
            f_to_channel_1, f_to_channel_2, f_to_channel_3,
            f_list_top, f_list_low,
            f_list_first,
        );
        // let ws_1 = f_list_first.await;
        let mut f = File::create(file_path.as_ref().with_extension("xlsx"));
        dbg!(Instant::now());
        f.set_sheet(ws_1, "Summary");
        f.set_sheet(ws_top, "Верхний двигатель");
        f.set_sheet(ws_low, "Нижний двигатель");
        f.save();
        dbg!(Instant::now());
    };
    f.await;
}


#[test]
fn test_convert_csv_raw_to_excel() {
    use crate::convert::{stream::*, iterator::*};
    use futures::future::join;

    let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_03_09_2021 11_58_30";
    if let Some(values) = super::csv::read_values(&format!("{}.csv", file_path)) {
        let values = raw_to_elk(values);
        let lines = values_to_line(futures::stream::iter(values));
        let lines = values_line_to_simple(lines);
        let f = write_file(file_path, lines);
        futures::executor::block_on(f);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn convert_csv_to_excel() {
    use crate::async_channel::*;
    use crate::convert::{stream::*, iterator::*};
    use futures::join;

    let dir = "/home/user/.local/share/graphicmodbus/log/values/csv_raw";
    let files = [
        "2022_03_22-17_17_18",
//             "2022_02_24-17_01_08",
    ];

    for name in files {
        let file_path = format!("{}/{}", dir, name);

        dbg!(format!("{}.csv", file_path));

        if let Some(values) = super::csv::read_values(&format!("{}.csv", file_path)) {
            let values = fullvalue_to_elk(values);

            let lines = values_to_line(futures::stream::iter(values));

            let (s, l_top) = broadcast(500);
            let f_to_channel = lines.map(|l| Ok(l)).forward(s);
            let l_low = l_top.clone();
//                 let l_low = lines;

            let l_top = crate::stat_info::simple::filter_half_top(l_top);
            let l_low = crate::stat_info::simple::filter_half_low(l_low);

            let f_top = write_file(file_path.clone() + "_top.xlsx", l_top);
            let f_low = write_file(file_path + "_low.xlsx", l_low);

            let f = async {
                let _ = join!(
                    f_to_channel,
                    f_top,
                    f_low
                );
            };

            // futures::executor::block_on(f);
            f.await;
        }
    }
    // assert!(false);
}

pub async fn convert_csv_to_excel_2() {
    use crate::async_channel::*;
    use crate::convert::{stream::*, iterator::*};
    use crate::stat_info::simple::*;
    use futures::join;

    let dir = "/home/user/.local/share/graphicmodbus/log/values/csv_raw";
    let files = [
        // "2022_03_22-17_17_18",
        // "2022_03_22-17_05_31",
        "2022_03_29-13_58_12",
//             "2022_02_24-17_01_08",
    ];

    for name in files {
        let file_path_ = format!("{}/{}", dir, name);
        let file_path = format!("{}.csv", file_path_);

//             dbg!(format!("{}.csv", file_path));

        let half = |path| {
            dbg!(&path);
            let values = crate::files::csv::read_values(path).unwrap();
            let values = fullvalue_to_elk(values);
            values_to_line(futures::stream::iter(values))
        };
        let f = write_file_3(file_path.clone(), half(file_path.clone()));
        // futures::executor::block_on(f);
        f.await;
    }
    // assert!(false);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_convert_csv_to_excel_2() {
    convert_csv_to_excel_2().await;
}
#[test]
fn test_block_convert_csv_to_excel_2() {
    futures::executor::block_on(convert_csv_to_excel_2());
}