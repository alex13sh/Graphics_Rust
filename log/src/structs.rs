#![allow(dead_code, unused_variables, unused_imports)]

use std::fs::File;
pub use std::path::PathBuf;
use std::io::prelude::*;
use std::time::Duration;
use std::collections::HashMap;

pub struct Converter {
    input_path_csv: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) file_name: String,
}

pub struct InputValues {
    pub(crate) converter: Option<Converter>,
    name_hash: Vec<(String, String)>,
    values: Vec<crate::LogValue>
}

pub struct OutputValues {
    pub(crate) converter: Option<Converter>,
    info: TableInfo,
    pub fields: Vec<String>,
    pub values: ValuesF,
}

struct TableInfo {
    step_sec: f32, // Шаг времени
    count: u32, // Кол-во значений
}

#[derive(Debug, Clone)]
pub struct TableState {
    pub time_work: f32, // время работы // count * step_sec = time
    pub time_acel: f32, // Время разгона
    pub hz_max: u32, // ValueHZ
    pub vibro_max: f32,
    pub hz_vibro: u32, // Зона вибрации
    pub tok_max: u32,
    pub temps_min_max: Vec<(String, (f32, f32))>,
}

impl Converter {
    pub fn new(input_path_csv: PathBuf, output_path: PathBuf) -> Self {
        Converter {
            input_path_csv: input_path_csv,
            output_path: output_path,
            file_name: String::new(),
        }
    }

    pub fn from_log_values(self, values: Vec<crate::LogValue>) -> InputValues {
        InputValues {
            converter: Some(self),
            .. InputValues::from_log_values(values)
        }
    }
    pub fn read_file(mut self, file_name: &str, reader: fn(&PathBuf) -> Vec<crate::LogValue>) -> InputValues {
        self.file_name = file_name.to_owned();
        let cur_path = self.input_path_csv.join(file_name.to_owned()+".csv");
        InputValues {
            converter: Some(self),
            .. InputValues::from_log_values(reader(&cur_path))
        }
    }
    pub fn read_file_opt(mut self, file_name: &str, reader: fn(&PathBuf) -> Option<Vec<crate::LogValue>>) -> Option<InputValues> {
        self.file_name = file_name.to_owned();
        let cur_path = self.input_path_csv.join(file_name.to_owned()+".csv");
        let v = InputValues {
            converter: Some(self),
            .. InputValues::from_log_values(reader(&cur_path)?)
        };
        Some(v)
    }
    pub fn output_file(output_path: PathBuf, file_name: &str) -> Self {
        Converter {
            input_path_csv: PathBuf::new(),
            output_path: output_path,
            file_name: file_name.to_owned(),
        }
    }
    pub fn get_output_file_path(&self) -> PathBuf {
        self.output_path.join(self.file_name.clone())
    }
}

impl InputValues {
    pub fn from_log_values(values: Vec<crate::LogValue>) -> InputValues {
        InputValues {
            converter: None,
            values: values,
            name_hash: Vec::new(),
        }
    }
    
    pub fn fields(mut self, name_hash: Vec<(&str, &str)>) -> Self {
        self.name_hash = name_hash.into_iter()
            .map(|(name, hash)| (name.into(), hash.into()))
            .collect();
        self
    }
    
    pub fn make_values_3(self, step_sec: Duration) -> OutputValues {
        let values = &self.values;
        let name_hash = &self.name_hash;

        let dt_finish = values.last().unwrap().date_time;
        let dt_start = values.first().unwrap().date_time;
        let dt_dlt =  dt_finish - dt_start;
        let cnt = values.iter().filter(move |v| v.hash == "4bd5c4e0a9").count();
        dbg!(&dt_dlt, &cnt);
        let dt_dlt: Duration = dt_dlt.to_std().unwrap();
        let step_ms = step_sec.as_millis();
        let cnt = dt_dlt.as_millis() / step_ms;
        let cnt = cnt as u32;
        dbg!(&cnt);

    //Vec::with_capacity(name_hash.len())
        let mut values_f32 = ValuesF::from_size(name_hash.len(), (cnt+2) as usize, -13.37);
        let fields: HashMap<_, _> = name_hash.iter().zip(0..).map(|((_,hash), i)| ( hash.to_owned(), i)).collect();
        dbg!(&fields);
    //     let step_ms = step_sec.as_millis() as i64;
        for v in values {
            let i = (v.date_time.timestamp_millis() - dt_start.timestamp_millis())/(step_ms as i64);
            if let Some(f) = fields.get(&v.hash) {
    //             dbg!(i, *f);
                values_f32.0[i as usize][*f as usize] = v.value;
            }
        }

//         let mut values: Vec<Vec<String>> = std::iter::repeat(vec![String::from("");name_hash.len()+1]).take(cnt as usize+1).collect();
        let step_sec_f = step_sec.as_secs_f32();
//         let values_str: Vec<_> = values_f32.into_iter().zip(0..)
//             .map(|(row, i)| {
//                 let time = i as f32 * step_sec_f;
//                 let mut rows = Vec::new();
//                 rows.push(format!("{:.1}", time).replace(".", ","));
//                 rows.extend(row.into_iter().map(|v|
//                     if v == -13.37 { String::new()}
//                     else { format!("{:.2}", v).replace(".", ",")}
//                 ));
//                 rows
//             }).collect();
//         let values_str = values_f32.to_string();

        let fields: Vec<_> = name_hash.iter().map(|(name,_)| name.to_owned()).collect();

        OutputValues {
            converter: self.converter,
            info: TableInfo {
                count: cnt,
                step_sec: step_sec.as_secs_f32(),
            },
            fields: fields,
//             values_str: values_str,
            values: values_f32,
        }
    }
}

impl OutputValues {
    pub fn get_state(&self) -> TableState {
        let time_work = self.info.count as f32 * self.info.step_sec;
        
        let column_hz = self.fields.iter().position(|s| s=="Скорость").unwrap();
        let column_time = self.fields.iter().position(|s| s=="time").unwrap();
        let column_vibro = self.fields.iter().position(|s| s=="Вибродатчик").unwrap();
        
        let (hz_max, time_acel) = self.values.0.iter().rev()
            .map(|row| (row[column_hz] as u32, row[column_time]))
            .max_by_key(|v| v.0).unwrap();
        
        let (vibro_max, hz_vibro) = self.values.0.iter().rev()
            .map(|row| (row[column_vibro], row[column_hz] as u32))
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap()).unwrap();
          
        let col_temp_1 = self.fields.iter().position(|s| s == "Температура ротора").unwrap();
        let mut temps_min_max = Vec::new();
        let row_first = &self.values.0.first().unwrap();
        let row_last = &self.values.0.last().unwrap();
        
        let column_tok = self.fields.iter().position(|s| s=="Ток").unwrap();
        let tok_max = self.values.0.iter()
            .map(|row| row[column_tok] as u32)
            .max().unwrap();

        for col in col_temp_1..self.fields.len() {
            temps_min_max.push((
                self.fields[col].clone(),
                (row_first[col], row_last[col])
            ));
        }
        
        TableState {
            time_work: time_work,
            time_acel: time_acel,
            hz_max: hz_max,
            hz_vibro: hz_vibro, // Вибро Зона
            vibro_max: vibro_max,
            tok_max: tok_max,
            temps_min_max: temps_min_max,
        }
    }

    pub fn get_state_build(&self) -> TableStateBuild {
        TableStateBuild::new(self)
    }

    pub fn fill_empty(mut self) -> Self {
        self.values.fill_empty();
        self
    }
    pub fn insert_time_f32(mut self) -> Self {
        self.fields.insert(0, "time".into());
        let info = &self.info;
        self.values.insert_column(0, 
            (0..).map(|v| v as f32 * info.step_sec));
        self
    }
    
    fn insert_speed(&mut self) {
        let column_hz = self.fields.iter().position(|s| s=="Скорость").unwrap();
        let speed : Vec<_> = self.values.0
            .iter().map(|row| row[column_hz]/1000.0).collect();
        self.values.insert_column(column_hz+1, speed.into_iter());
        self.fields.insert(column_hz+1, "Speed".into());
    }
    pub fn shift_vibro(mut self) -> Self {
        let column_vibro = self.fields.iter().position(|s| s=="Вибродатчик").unwrap();
        let shift = 0.5 / self.info.step_sec;
        let shift = shift as usize;
        let count = self.info.count as usize;
        for i in shift..count {
            let col_i = self.values.0[i][column_vibro];
            let row_s = &mut self.values.0[i-shift];
            let col_s = &mut row_s[column_vibro];
            *col_s = col_i;
        }
        self
    }
    pub fn convert_davl(mut self) -> Self {
        let col_davl = self.fields.iter().position(|s| s=="Давление масла на выходе маслостанции").unwrap();
        for v in self.values.0.iter_mut()
            .map(|row| &mut row[col_davl]) {
            *v = *v * 1.7;
        }
        self
    }
    
    pub fn write_csv(self) -> crate::MyResult {
        let conv = &self.converter.ok_or("Converter is empty")?;
        let info = &self.info;
        let new_path = conv.output_path
            .join(format!("{}_filter_{}.csv", conv.file_name, info.step_sec));
        use std::fs::OpenOptions;
        let mut wrt = ::csv::WriterBuilder::new()
            .delimiter(b';')
    //         .from_writer(file);
            .from_path(new_path)?;
        wrt.write_record(&self.fields).unwrap();

        let values_str = self.values.to_string();
        for s in values_str {
            if !s[0].is_empty() {
                wrt.write_record(&s)?;
            }
        }
        Ok(())
    }

    pub fn write_excel(mut self) -> crate::MyResult<PathBuf> {
//         use umya_spreadsheet::*;
        use excel::*;
        
//         self.shift_vibro();
//         self.values.fill_empty();
//         self.insert_time_f32();
        self.insert_speed();
        
        // self.convert_davl();
        
        let conv = &self.converter;
        let conv = conv.as_ref().ok_or("Converter is empty")?;
        let info = &self.info;
        
        let new_path = conv.output_path
            .join(format!("{}({}).xlsx", conv.file_name, info.step_sec));
        let mut book = umya_spreadsheet::new_file();
//         let sht = book.new_sheet("Лог")?;
        let sht = book.get_sheet_by_name_mut("Sheet1")?;

        let mut values_str = self.values.to_string();
//             values_str.insert_column(0, (0..info.count).map(|v| {
//                 let v = v as f32 *info.step_sec;
//                 format!("{:.1}", v)
//             }));

        for (f, col) in self.fields.iter().zip(1..) {
            sht.set_cell_value(col, 1, f);
        }

        for (s, row) in values_str.into_iter()
            .zip(2..) {

            sht.set_row_values(row, &s);
        }
        
//         sht.draw_line_serials(&[2, 3, 5], info.count as usize);
        
//         let sht = book.new_sheet("Инфо")?;
        
        let state = self.get_state();
        sht.write_state((14,2), state);
        
        let _ = umya_spreadsheet::writer::xlsx::write(&book, &new_path);
        Ok(new_path)
    }

}

#[derive(Default)]
pub struct TableStateFields {
    pub time_work: Option<f32>, // время работы // count * step_sec = time
    pub time_work_before_after_6_000: Option<(f32, f32)>, // время работы до и после 6 тыс
    pub time_work_hz: Option<Vec<f32>>, // время работы на разных частотах кратном 1_000

    pub time_acel: Option<f32>, // Время разгона
    pub hz_max: Option<u32>, // ValueHZ
    pub vibro_max: Option<f32>,
    pub hz_vibro: Option<u32>, // Зона вибрации
    pub tok_max: Option<u32>,
    pub temps_min_max: Option<Vec<(String, (f32, f32))>>,
}

pub struct TableStateBuild <'a> {
    values: &'a OutputValues,
    pub fields: TableStateFields,
}

impl <'a> TableStateBuild<'a> {
    fn new(values: &'a OutputValues) -> Self {
        TableStateBuild {
            values: values,
            fields: TableStateFields::default(),
        }
    }

    pub fn time_work(mut self) -> Self {
        let time_work = self.values.info.count as f32 * self.values.info.step_sec;
        self.fields.time_work = Some(time_work);
        self
    }
    pub fn time_work_before_after_6_000(mut self) -> Self {
        let column_hz = self.values.fields.iter().position(|s| s=="Скорость").unwrap();
        let column_time = self.values.fields.iter().position(|s| s=="time").unwrap();

        let mut time_work_before_after_6_000: (f32, f32) = (0.0, 0.0);
        let mut itr = self.values.values.0.iter();
        if let Some((hz, time_before_6_000)) = itr
            .map(|row| (row[column_hz] as u32, row[column_time]))
            .find(|(hz, _)| *hz>6_000) {

            time_work_before_after_6_000.0 = time_before_6_000;
        } else {
            if let Some(time_work) = self.fields.time_work {
                time_work_before_after_6_000.0 = time_work;
            } else {
                self = self.time_work();
                time_work_before_after_6_000.0 = self.fields.time_work.unwrap();
            }
        }

        self.fields.time_work_before_after_6_000 = Some(time_work_before_after_6_000);
        self
    }

    pub fn time_work_hz(mut self) -> Self {
        let column_hz = self.values.fields.iter().position(|s| s=="Скорость").unwrap();
        let column_time = self.values.fields.iter().position(|s| s=="time").unwrap();

//         for row in self.values.values.0.iter() {
//             let hz = row[column_hz] as u32;
//             let time = row[column_time];
//
//         }

        use itertools::Itertools;
        let mut time_work_hz = vec![0_f32; 24];
        let arr: Vec<_> = self.values.values.0.iter()
                .map(|row| (row[column_hz] as u32, row[column_time]))
                .collect();

        for (hz_1000, mut group) in arr.into_iter()
                .group_by(|(hz, _time)| *hz / 1_000).into_iter() {

            let time_first = group.next();
            let time_last = group.last();
            let time_work = time_first.zip(time_last)
                .map(|(t1, t2)| t2.1 - t1.1).unwrap_or(0_f32);

            time_work_hz[hz_1000 as usize] += time_work;
        }
        self.fields.time_work_hz = Some(time_work_hz);
        self
    }
}

mod excel {
    use umya_spreadsheet::structs::*;
    pub trait MyCell {
        fn set_cell_value<S: Into<String>>(&mut self, col: usize, row: usize, value: S);
        fn set_row_values(&mut self, row: usize, values: &[String]);
    }
    impl MyCell for Worksheet {
        fn set_cell_value<S: Into<String>>(&mut self, col: usize, row: usize, value: S) {
            self.get_cell_by_column_and_row_mut(col, row).set_value(value);
        }
        fn set_row_values(&mut self, row: usize, values: &[String]) {
            for (v, col) in values.into_iter().zip(1..) {
                self.get_cell_by_column_and_row_mut(col, row).set_value(v.clone());
            }
        }
    }
    
    pub trait WriteInfo {
        fn write_state(&mut self, pos: (usize, usize), state: super::TableState);
    }
    
    impl WriteInfo for Worksheet {
        fn write_state(&mut self, pos: (usize, usize), state: super::TableState) {
            let mut fields = Vec::new();
            fields.push(("Время работы (сек)", state.time_work.to_string()));
            fields.push(("Время разгона (сек)", state.time_acel.to_string()));
            fields.push(("Обороты двигателя (об/мин)", state.hz_max.to_string()));
            fields.push(("Максимальная вибрация", state.vibro_max.to_string()));
            fields.push(("Зона вибрации (об/мин)", state.hz_vibro.to_string()));
            fields.push(("Максимальный ток", state.tok_max.to_string()));
            for (f, row) in fields.into_iter().zip((pos.1+1)..) {
                self.set_cell_value(1+pos.0, row, f.0);
                self.set_cell_value(2+pos.0, row, f.1);
            }
            for (f, row) in state.temps_min_max.into_iter().zip((pos.1+8)..) {
                self.set_cell_value(pos.0+1, row, f.0);
                self.set_cell_value(pos.0+2, row, format!("{:.2}", f.1.0));
                self.set_cell_value(pos.0+3, row, format!("{:.2}", f.1.1));
            }
        }
    }
    
    pub trait Graphic {
        fn draw_line_serials(&mut self, columns: &[usize], rows: usize);
    }
    
    impl Graphic for Worksheet {
        fn draw_line_serials(&mut self, columns: &[usize], rows: usize) {
            use drawing::{charts, spreadsheet::*, };
            let mut celanch = TwoCellAnchor::default();
//             let mut shape = Shape::default();
//             let mut anchor = Anchor::default();
//             anchor.set_left_column(10);
//             anchor.set_top_row(5);
//             shape.set_anchor(anchor);
//             celanch.set_shape(shape);
            celanch.get_from_marker_mut()
                .set_col(10).set_row(5);
            celanch.get_to_marker_mut()
                .set_col_off(10)
                .set_row_off(10)
                .set_col(10)
                .set_row(10);
            
            let mut graph_frame = GraphicFrame::default(); 
            let graph = graph_frame.get_graphic_mut().get_graphic_data_mut();
            let chart = graph.get_chart_space_mut()
                .get_chart_mut().get_plot_area_mut();
            
            let mut linechart = charts::LineChart::default();
            let mut areacharts = Vec::new();
            for col in columns {
                let mut adr = Address::default();
                adr.set_sheet_name(self.get_title());
                adr.set_address(format!("${0}${1}:${0}${2}", solumnt_to_string(*col),2,rows));
//                 dbg!(&adr);
                let mut values = charts::Values::default();
                values.get_number_reference_mut().get_formula_mut()
                    .set_address(adr);
                let mut areachart = charts::AreaChartSeries::default();
                areachart.set_values(values);
                areacharts.push(areachart)
            }
            
            linechart.set_area_chart_series(areacharts);
            chart.set_line_chart(linechart);
            celanch.set_graphic_frame(graph_frame);
            dbg!(&celanch);
            self.get_worksheet_drawing_mut().add_two_cell_anchor_collection(celanch);
        }
    }
    fn solumnt_to_string(column: usize) -> String {
        assert!(column <= 26);
        ((('A' as u8)+(column-1) as u8) as char).to_string()
    }
}

pub use inner::*;
mod inner {

    pub type ValuesF = ValuesMat<f32>;
    pub type ValuesS = ValuesMat<String>;

    pub struct ValuesMat<T>(pub Vec<Vec<T>>);

    impl <T> ValuesMat <T>
    where T: std::clone::Clone
    {
        pub fn from_size(cols: usize, rows: usize, value: T) -> Self {
            let v = std::iter::repeat(vec![value;cols]).take(rows+1).collect();
            ValuesMat(v)
        }
        pub fn insert_column(&mut self, col: usize, values: impl Iterator<Item=T>) -> &mut Self {
            if self.0.is_empty() {return self;}
            let rows = self.0[0].len();
//             let v = self.0.iter()
//                 .zip(values)
//                 .map(|(row, v)| {
//                     let mut row_new = Vec::with_capacity(rows+1);
//                     row_new.extend(row.into_iter());
//                     row_new.insert(col, v);
//                     row_new
//                 }).collect();
            for (row, v) in self.0.iter_mut().zip(values) {
                row.insert(col, v);
            }
//             ValuesMat(v)
//             self.0 = v;
            self
        }
    }

    impl ValuesMat<f32> {
        pub fn to_string(&self) -> ValuesMat<String> {
            let v = self.0.iter()
//             .zip(0..)
//             .map(|(row, i)| {
            .map(|row| {
                let mut rows = Vec::new();
//                 let time = i as f32 * step_sec_f;
//                 rows.push(format!("{:.1}", time).replace(".", ","));
                rows.extend(row.into_iter().map(|&v|
                    if v == -13.37 { String::new()}
                    else { format!("{:.2}", v)}
                ));
                rows
            }).collect();
            ValuesMat(v)
        }
        pub fn fill_empty(&mut self) {
            for y in 1..self.0.len() {
                for x in 0..self.0[y].len() {
                    if self.0[y][x] == -13.37 {
                        self.0[y][x] = self.0[y-1][x];
                    }
                }
            }
        }
    }

    impl <T> IntoIterator for ValuesMat<T> {
        type Item = <Vec<Vec<T>> as IntoIterator>::Item;
        type IntoIter = <Vec<Vec<T>> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    pub struct MyZip <T, U>
    where T: Iterator<Item=U>
    {
        vec: Vec<T>
    }
    // impl <T, U> MyZip<T, U>
    // where
    //     T: IntoIterator<Item=U>
    impl <T, U> MyZip<T, U>
    where T: Iterator<Item=U>
    {
        pub fn new(v: Vec<T>) -> Self
        {
            MyZip {
                vec:v.into_iter()
                .fold(Vec::new(), |mut v, i| {
                    v.push(i);
                    v
                })
            }
        }
    }

    impl <T, U> Iterator for MyZip<T, U>
    where T: Iterator<Item=U>
    {
        type Item = Vec<U>;
        fn next(&mut self) -> Option<Self::Item> {
            self.vec.iter_mut()
                .map(|v| v.next())
                .collect()
        }
    }
    pub fn myzip<T, U>(vec: Vec<T>) -> MyZip<T::IntoIter, U>
    where
        T: IntoIterator<Item=U>
    {
        MyZip::new(vec.into_iter()
            .map(|v| v.into_iter())
            .collect())
    }

//     fn convert_cols_to_rows<T>(cols: Vec<Vec<T>>) -> Vec<Vec<T>> {
//         let mut itr = Box<dyn Iterator>;
//         for c in cols {
//             itr = itr.interleave(c);
//         }
//         itr.chunks(cols.len()).collect()
//     }

    use std::iter::Peekable;
    pub struct InsertByStep <T, I, ITP, P>
    where I: Iterator<Item=T>,
        ITP: Iterator<Item=(T,usize)>,
        P: Iterator<Item=usize>
    {
        a: Peekable<ITP>, b: I,
        pos: Peekable<P>,
    }

    impl <T, I, ITP, P> Iterator for InsertByStep <T, I, ITP, P>
        where I: Iterator<Item=T>,
        ITP: Iterator<Item=(T,usize)>,
        P: Iterator<Item=usize>
    {
        type Item = T;
        fn next(&mut self) -> Option<Self::Item> {
            let pos_a = self.a.peek()?.1;
            let pos_p = self.pos.peek()?;
            if pos_a == *pos_p {
                self.pos.next();
                self.b.next()
            } else {
                self.a.next().map(|a| a.0)
            }
        }
    }

    pub fn insert_by_step<T, I, P>(a: I, b: I, pos: P) -> InsertByStep<T, I, impl Iterator<Item=(T,usize)>, P>
    where I: Iterator<Item=T>,
        P: Iterator<Item=usize>
    {
        InsertByStep {
            a: a.zip(0..).peekable(), b: b,
            pos: pos.peekable(),
        }
    }


    #[test]
    fn test_iner_step() {
//         use itertools::Itertools;
        let a = (0..10);
        let b = (20..25);

        let mut c = insert_by_step(a, b, (0..).step_by(2));

        assert_eq!(c.next().unwrap(), 20);
        assert_eq!(c.next().unwrap(), 0);
        assert_eq!(c.next().unwrap(), 1);
        assert_eq!(c.next().unwrap(), 21);
        assert_eq!(c.next().unwrap(), 2);
        assert_eq!(c.next().unwrap(), 3);
        assert_eq!(c.next().unwrap(), 22);
        assert_eq!(c.next().unwrap(), 4);
        assert_eq!(c.next().unwrap(), 5);
        assert_eq!(c.next().unwrap(), 23);
        assert_eq!(c.next().unwrap(), 6);
        assert_eq!(c.next().unwrap(), 7);
        assert_eq!(c.next().unwrap(), 24);
        assert_eq!(c.next().unwrap(), 8);
        assert_eq!(c.next().unwrap(), 9);
    }
}

// Всё ненужное

impl InputValues {
    pub fn make_values_1(self, step_sec: Duration) -> crate::MyResult<OutputValues> {
        let values = &self.values;
        let name_hash = &self.name_hash;
        let step_sec = step_sec.as_secs_f32();

        let dt_dlt = values.last().unwrap().date_time - values.first().unwrap().date_time;
        let cnt = values.iter().filter(move |v| v.hash == "4bd5c4e0a9").count();
        dbg!(&dt_dlt, &cnt);
        let step_ms_100 = (step_sec * 100.0) as i32;
        let dt_dlt = dt_dlt * 100;
        let stp = cnt as f32 / (dt_dlt /step_ms_100).num_seconds() as f32;
        dbg!(&stp);
        let stp = stp.round() as usize;
        if stp == 0 {dbg!(&stp);return Err("Step = 0".into());}

        let lst: Vec<_> = name_hash.into_iter().map(|(name, hash)| {
            values.iter()
                .filter(move |v| &v.hash == hash)
                .zip(0..cnt)
                .map(move |(v,i)|
                    v.value
                ).step_by(stp)
        }).collect();

        let lst : Vec<_> = MyZip::new(lst)
            .collect();
    //     dbg!(&lst);
        let cnt = lst.len();
        let fields: Vec<_> = name_hash.iter().map(|(name,_)| name.to_owned()).collect();

        Ok(OutputValues {
            converter: self.converter,
            info: TableInfo {
                count: cnt as u32,
                step_sec: step_sec,
            },
            fields: fields,
            values: ValuesMat(lst),
        })
    }

    pub fn make_values_2(self, step_sec: Duration) -> crate::MyResult<OutputValues> {
        let values = &self.values;
        let name_hash = &self.name_hash;

        let dt_dlt = values.last().unwrap().date_time - values.first().unwrap().date_time;
//         dbg!(&dt_dlt);
        use std::collections::BTreeMap;
    //     let map_values = BTreeMap::new();

        let felds: Vec<_> = name_hash.iter().map(|(name,_)| name.to_owned()).collect();
        let lst: Vec<_> = name_hash.into_iter().map(|(name, hash)| {
            let dt_value: BTreeMap<_, _> = values.iter()
                .filter(move |v| &v.hash == hash)
                .map(move |v| {
                    let dt = (v.date_time+crate::Duration::hours(3)).format("%H:%M:%S").to_string();
                    (dt, v.value)
                }).collect();
            let val: Vec<_> = dt_value.values().cloned().collect();
            val
        }).collect();

        let lst : Vec<_> = myzip(lst)
            .collect();
        let cnt = lst.len();
        let fields: Vec<_> = name_hash.iter().map(|(name,_)| name.to_owned()).collect();

        Ok(OutputValues {
            converter: self.converter,
            info: TableInfo {
                count: cnt as u32,
                step_sec: step_sec.as_secs_f32(),
            },
            fields: fields,
            values: ValuesMat(lst),
        })
    }
}
