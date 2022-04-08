#![allow(dead_code, unused_imports)]
#![feature(map_first_last)]

#[cfg(feature = "plotters")]
mod plotter_values;
mod file;

#[cfg(feature = "gui")]
mod gui;

use std::collections::HashMap;

// type LineSeries = log_new::value::ValuesSeries<f32>;
type DateTime = log_new::DateTimeFix;
type ValuesLineRecv = log_new::Reciv<log_new::value::ValuesLine<log_new::value::Value>>;
type DatePoint = log_new::value::ValueDate<f32>;
use log_new::date_time_now;



#[derive(Debug)]
pub struct LineSeries {
    name: String,
    graphic_second: bool,
    // color: iced_native::Color,
    points: Vec<DatePoint>,
    // min_max_value: Option<(f32, f32)>,
}

impl LineSeries {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            graphic_second: false,
            points: Vec::new(),
        }
    }
    pub fn with_graphic_second(mut self, is_second: bool) -> Self {
        self.graphic_second = is_second;
        self
    }
    pub fn set_graphic_second(&mut self, is_second: bool) -> &mut Self {
        self.graphic_second = is_second;
        self
    }
    pub fn is_graphic_second(&self) -> bool {
        self.graphic_second
    }
    pub fn addPoint(&mut self, point: DatePoint) {
        self.points.push(point);
    }
    pub fn get_points(&self) -> &[DatePoint] {
        &self.points
    }
    pub fn convert_to_i32(&mut self) {
        for point in &mut self.points {
            if point.value > 3200.0 {
                point.value = 6553.5 - point.value;
             }
        }
    }
//     fn calc_min_max_value(&mut self) -> Option<(f32, f32)> {
//         let min = self.points.iter()
//             .min_by(|a, b| a.value.partial_cmp(&b.value).unwrap())?.value;
//         let max = self.points.iter()
//             .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())?.value;
//         self.min_max_value = Some((min, max));
//         self.min_max_value
//     }
}


#[derive(Debug)]
struct ViewPort {
    start: DateTime,
    end: DateTime,
    min_value: f32,
    max_value: f32,
}

impl ViewPort {
    fn calc_procent_horizont(&self, dt: DateTime) -> f32 {
        (dt.timestamp_millis() - self.start.timestamp_millis()) as f32 / 
        (self.end.timestamp_millis() - self.start.timestamp_millis()) as f32
    }
    fn calc_procent_vertical(&self, value: f32) -> f32 {
        (self.max_value - value) /
        (self.max_value - self.min_value)
    }
//     fn calc_point(&self, p: &DatePoint, size: Size) -> (f32, f32) {
//         let y = self.calc_procent_vertical(p.value)   * size.height;
//         let x = self.calc_procent_horizont(p.date_time)      * size.width;
//         (x, y)
//     }
    
    fn set_end(&mut self, end: DateTime) {
        let dlt = self.end - self.start;
        self.end = end;
        self.start = self.end - dlt;
    }
    
    fn get_slice_points<'a>(&self, points: &'a Vec<DatePoint>) -> &'a [DatePoint] {
        let points = &points[..];
        let i_start=match points.binary_search_by(|point| point.date_time.cmp(&self.start)) {Ok(pos)=>pos, Err(pos)=>pos};
        let i_end=match points.binary_search_by(|point| point.date_time.cmp(&self.end)) {Ok(pos)=>pos, Err(pos)=>pos};
        &points[i_start..i_end]
    }
}

// LineSeries iter into 
// impl Iterator for LineSeriesIter;
// type Item = iced::Point;

fn averge_iterator(points: &[DatePoint], max_points: usize) -> impl Iterator<Item=DatePoint> + '_ {
    points.chunks(points.len()/max_points+1).map(|points| {
        let sum_value = points.iter().fold(0_f32, |value, point| value + point.value);
        DatePoint {
            date_time: points.first().unwrap().date_time ,
            value: sum_value / points.len() as f32
        }
    })
}

#[test]
fn test() {
    
}
