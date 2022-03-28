   
pub mod csv {
    use super::inner::*;
    use csv::WriterBuilder;

    pub fn read_values<T>(file_name: impl AsRef<Path>) -> Option<impl Iterator<Item=T>>
    where T: for<'de> serde::Deserialize<'de>
    {
        
        let file = File::open(file_name).ok()?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_reader(file);
            
       let itr =  std::iter::from_fn(move || {
            rdr.deserialize()
                .filter_map(|res| res.ok())
            }.next()
        );
        Some(itr)
    }
    
    pub fn write_values<T>(file_name: impl AsRef<Path>, values: impl Iterator<Item=T>) -> crate::MyResult 
    where T: serde::Serialize
    {
        let file = File::create(file_name)?;
        let mut wrt = csv::WriterBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_writer(file);
        
        for value in values {
            wrt.serialize(value)?;
        }
        
        Ok(())
    }
    
    pub fn write_values_async<T>(file_name: impl AsRef<Path>, values: impl Stream<Item=T> ) -> crate::MyResult<impl Future<Output=()>>
    where T: serde::Serialize
    {
        let file = File::create(file_name)?;
        let wrt = csv::WriterBuilder::new()
            .has_headers(true)
            .delimiter(b';')
            .from_writer(file);
        
//         while let Some(value) = values.next().await {
//             wrt.serialize(value)?;
//         }
        let f = async {
            let _ = values.fold(wrt, |mut wrt, v| async {
                wrt.serialize(v).unwrap();
                wrt
            }).await;
        };
        Ok(f)
    }
    
    #[test]
    fn test_read_write_csv() {
        use crate::value::raw::*;
        use crate::value::ValueDate;
        use crate::async_channel::*;
        let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_04_08_2021__12_27_52_673792376";
        if let Some(lines) = read_values(&format!("{}.csv", file_path)) {
            
            let lines = lines.map(|v: ValueDate<ValueOld>| 
                ValueDate {
                    date_time: v.date_time,
                    value: Value::from(v.value),
                }
            );
            
            let (mut s1, r1) = broadcast(10);
            let r2 = r1.clone();
//             write_values("/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_04_08_2021__12_27_52_673792376_sync.csv", lines).unwrap();
            let f0 = async move {
                for v in lines {
                    // dbg!(&v);
                    s1.send(v).await.unwrap();
                }
            };
            let f1 = write_values_async(format!("{}_async_1.csv", file_path), r1).unwrap();
            let f2 = write_values_async(format!("{}_async_2.csv", file_path), r2).unwrap();
            
            let _ = futures::executor::block_on( futures::future::join3(f0, f1, f2) );
            
        } else {
            assert!(false);
        }
//         assert!(false);
    }
    
    fn get_raw_files_iter(dir: &str) -> impl Iterator<Item=PathBuf> {
        let path = get_file_path(dir);
        dbg!(&path);
        let paths = std::fs::read_dir(path).unwrap();
        let iter = paths.filter_map(|res| res.ok())
        .map(|dir| dir.path())
        .filter(|path|
            if let Some(ext) = path.extension() {
                ext == "csv"
            } else {false}
        );
//         res.sort_by_key(|p| p.metadata().unwrap().modified().unwrap());
        iter
    }

    fn file_name2date_time(file_name: &str) -> Option<DateTimeFix> {
        dbg!(file_name);
        let file_name = &format!("{} +0300", file_name);
        let res = DateTimeFix::parse_from_str(file_name, "value_%d_%m_%Y %H_%M_%S.csv %z")
        .or_else(|_|
            DateTimeFix::parse_from_str(file_name, "value_%d_%m_%Y__%H_%M_%S.csv %z")
        ).or_else(|_|
            DateTimeFix::parse_from_str(file_name, "value_%d_%m_%Y__%H_%M_%S_%.f.csv %z")
        ).or_else(|_|
            DateTimeFix::parse_from_str(file_name, "+value_%d_%m_%Y__%H_%M_%S_%.f_filter_0.1.csv %z")
        )
        .inspect_err(|e| {dbg!(e);})
        .ok();
        dbg!(&res);
        res
    }

    #[test]
    fn test_convert_raw_to_elk() {
        use crate::value::raw::*;
        use crate::value::ValueDate;
        let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_04_08_2021__12_27_52_673792376";
        if let Some(values) = read_values(&format!("{}.csv", file_path)) {
            let values = crate::convert::iterator::raw_to_elk(values);
            write_values(&format!("{}_elk.csv", file_path), values).unwrap();
        }
//         assert!(false);
    }

    #[test]
    fn test_converts_raw_to_elk() {
        use crate::value::raw::*;
        use crate::value::ValueDate;
        use rayon::prelude::*;
        use rayon::iter::ParallelBridge;
        use rayon::prelude::ParallelIterator;

        let dir_elk = get_file_path("log/values/csv/");
        // for file_path in  {
        get_raw_files_iter("log/values/csv_raw/")
            // .par_bridge()

        .filter_map(|file_path| {
            Some((
                file_name2date_time(file_path.file_name().unwrap().to_str().unwrap())?,
                read_values(file_path)?
            ))
        })
        .map(|(dt, values)| (dt, crate::convert::iterator::raw_to_elk(values)))
        .map(|(dt, values)| (dt, crate::convert::iterator::value_date_shift_time(values, 3)) )
        .for_each(
            |(dt, values)| {
                write_values(dir_elk
                    .join(date_time_to_string_name_short(&dt)).with_extension("csv"), 
                    values).unwrap();
            }
        );
        assert!(false);
    }
    
    #[test]
    fn test_convert_to_table() {
        let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_03_09_2021 11_58_30";
        if let Some(values) = read_values(&format!("{}.csv", file_path)) {
            use crate::convert::stream::*;
            let values = crate::convert::iterator::raw_to_elk(values);
            let lines = values_to_line(futures::stream::iter(values));
            let lines = values_line_to_hashmap(lines);
            futures::executor::block_on( write_values_async(format!("{}_table.csv", file_path), lines).unwrap() );
//             assert!(false);
        }
    }
    
    #[test]
    fn test_values_line_diff() {
        let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_03_09_2021 11_58_30";
        if let Some(values) = read_values::<crate::value::LogValueRawOld>(&format!("{}.csv", file_path)) {
            use crate::convert::stream::*;
            let lines = values_to_line(futures::stream::iter(values));
            let values = values_from_line_with_diff(lines);
            futures::executor::block_on( write_values_async(format!("{}_diff.csv", file_path), values).unwrap() );
        }
    }
}

pub mod excel {
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

        pub fn first_sheet(&mut self, name: &'static str) -> Sheet<&mut Worksheet> {
            self.book.set_sheet_title(0, name).unwrap();
            let sht = self.book.get_sheet_mut(0);
            // sht.set_title(name);
            Sheet::from(
                sht
            )
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
            self.book.add_sheet(ws);
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
        ws: Sh
    }

    pub trait SheetInner {
        fn get_cell_by_column_and_row_mut(&mut self, col:usize, row:usize)-> &mut Cell;
        fn calculation_auto_width(&mut self);
    }

    impl SheetInner for Worksheet {
        fn get_cell_by_column_and_row_mut(&mut self, col:usize, row:usize)-> &mut Cell {
            self.get_cell_by_column_and_row_mut(col as u32, row as u32)
        }
        fn calculation_auto_width(&mut self) {
            self.calculation_auto_width();
        }
    }
    impl SheetInner for &mut Worksheet {
        fn get_cell_by_column_and_row_mut(&mut self, col:usize, row:usize)-> &mut Cell {
            <Worksheet as SheetInner>::get_cell_by_column_and_row_mut(self, col, row)
        }
        fn calculation_auto_width(&mut self) {
            <Worksheet as SheetInner>::calculation_auto_width(self);
        }
    }
    
    impl <Sh> From<Sh> for Sheet<Sh> 
        where Sh: SheetInner
    {
        fn from(v: Sh) -> Self {
            Self {
                ws: v
            }
        }
    }

    impl Sheet <Worksheet> {
        pub fn new() -> Self {
            let ws = Worksheet::default();
            // ws.set_title(name.into());
            Self {
                ws
            }
        }
    }

    // impl Sheet <&mut Worksheet> {
    //     pub fn new_2() -> (Worksheet, Self) {
    //         let mut ws = Worksheet::default();
    //         // ws.set_title(name.into());
    //         let s = Self {
    //             ws: &mut ws
    //         };
    //         (ws, s)
    //     }
    // }

    impl <Sh> Sheet <Sh> 
        where Sh: SheetInner
    {
        // pub fn new() -> Self 
        //     where Sh: Default
        // {
        //     let mut ws = Default::default();
        //     Self {
        //         ws: ws
        //     }
        // }
        pub async fn write_values(&mut self, pos: (usize, usize), values: impl Stream<Item=simple::ValuesMap> + std::marker::Unpin) {
            let mut values = values.enumerate().peekable();
            
            let l = if let Some(ref l) = std::pin::Pin::new(&mut values).peek().await {&l.1}
            else {return};

            self.ws.get_cell_by_column_and_row_mut(pos.0 + 0, pos.1 + 0)
                    .set_value("Время");
            let dt_start = l.date_time.clone();
            
            for (col, name) in l.values.keys().enumerate() {
                self.ws.get_cell_by_column_and_row_mut(pos.0 + col+1, pos.1).set_value(name);
            }
        
            // self.ws.calculation_auto_width();

            while let Some((row, l)) = values.next().await {
                let time = l.date_time.timestamp_millis() - dt_start.timestamp_millis();
                let time = (time as f32 / 100.0).round() / 10.0;
                self.ws.get_cell_by_column_and_row_mut(pos.0 + 0, pos.1 + row+1)
                    .set_value(time.to_string());
                for (col, v) in l.values.values().enumerate() {
                    self.ws.get_cell_by_column_and_row_mut(pos.0 + col+1, pos.1 + row+1).set_value(v);
                }
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
                sheet.write_values((1,1), l1),
                crate::stat_info::simple::calc(l2).fold(None, |_, s| async{Some(s)})
            ).await;
            dbg!("await");
            if let Some(stat) = stat {
                sheet.write_state((12,2), stat);
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
            let sht = Sheet::from(&mut ws);
            // dbg!(Instant::now());
            write_file_inner(lines_top_1, sht).await;
            dbg!(Instant::now());
            ws
        };

        let f_list_low = async move {
            let mut ws = Worksheet::default();
            let sht = Sheet::from(&mut ws);
            // dbg!(Instant::now());
            write_file_inner(lines_low_1, sht).await;
            dbg!(Instant::now());
            ws
        };

        /* let f_list_first = async move {
        //     let mut ws = Worksheet::default();
        //     let mut sht = Sheet::from(&mut ws);

        //     let lines_top_2 = filter_lines(lines_top_2, |sensor_name| {
        //         !sensor_name.starts_with("Температура")
        //     });
        //     let lines_low_2 = filter_lines(lines_low_2, |sensor_name| {
        //         !sensor_name.starts_with("Температура")
        //     });

        //     let lines_top_2 = values_simple_line_to_hashmap(lines_top_2);
        //     let lines_low_2 = values_simple_line_to_hashmap(lines_low_2);

        //     // let _ = join!(
        //         sht.write_values((1,1), lines_top_2).await;
        //         sht.write_values((10,1), lines_low_2).await;
        //     // );
        //     ws
        // }; */
        let f_list_first = async move {
            let mut ws = Worksheet::default();
            let mut sht = Sheet::from(&mut ws);

            let lines_top_2 = filter_half(lines_top_2);
            let lines_top_2 = filter_lines_map(lines_top_2, |sensor_name| {
                if !sensor_name.starts_with("Температура") {
                    Some(sensor_name.to_owned() + " (Верх.)")
                } else {
                    None
                }
            });
            
            // let lines_top_2 = lines_top_2.inspect(|l| {
            //     let lines_top_2 = &l.values;
            //     dbg!("lines_top_2", lines_top_2);
            // });

            let lines_low_2 = filter_half(lines_low_2);
            let lines_low_2 = filter_lines_map(lines_low_2, |sensor_name| {
                if !sensor_name.starts_with("Температура") {
                    Some(sensor_name.to_owned() + " (Ниж.)")
                } else {
                    None
                }
            });

            let lines = join_lines_2(lines_top_2, lines_low_2);

            // let lines = lines.inspect(|l| {
            //     dbg!("lines", l.values.iter().map(|v| v.sensor_name.clone()));
            // });

            let lines = values_simple_line_to_hashmap(lines);
            // dbg!(Instant::now());
            // let lines = lines.take(1);

            sht.write_values((1,1), lines).await;
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
            "2022_03_22-17_17_18",
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
}

pub mod invertor {

    #[test]
    fn config_csv_sort() {
        use crate::files::csv;
        let dir = "/home/user/.local/share/graphicmodbus/tables/save_invertor_top";
        let file_name = "22_11_2021 12_17_59.";

        let params = csv::read_values(format!("{}/{}.csv", dir, file_name)).unwrap();
        let params = crate::convert::iterator::invertor_parametrs_sort(params);
        csv::write_values(format!("{}/{}_sort.csv", dir, file_name), params).unwrap();
    }
}

mod inner {
    pub use futures::stream::{Stream, StreamExt};
    pub use std::path::{PathBuf, Path};
    pub use std::fs::File;
    pub use std::future::Future;
    pub use crate::utils::{get_file_path, DateTimeFix, date_time_to_string_name_short};
}
