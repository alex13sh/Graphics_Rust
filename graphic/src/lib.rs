#![allow(dead_code, unused_imports)]

use iced::{
    canvas::{
        self, Cache, Canvas, Cursor, Event, Frame, Geometry, Path, Stroke,
    },
    Svg, svg, Container,
    mouse, Color, Element, HorizontalAlignment, Length, Point, Rectangle,
    Size, Vector, VerticalAlignment,
};

use std::collections::HashMap;

// type LineSeries = log_new::value::ValuesSeries<f32>;
type DateTime = log_new::DateTimeFix;
type ValuesLineRecv = log_new::Reciv<log_new::value::ValuesLine<log_new::value::Value>>;
type DatePoint = log_new::value::ValueDate<f32>;
use log_new::date_time_now;

pub struct Graphic {
    series: HashMap<String, LineSeries>,
    view_port: ViewPort,
    dt_start: DateTime,
    
    grid_cache: Cache,
    lines_cache: Cache,
    plotters_svg: Option<svg::Handle>,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadLog()
}

impl Graphic {

    pub fn new() -> Self {
        let mut res = Self {
            series: HashMap::new(),
            view_port: ViewPort {
                end: date_time_now(),
                start: date_time_now() - log_new::utils::Duration::seconds(2*60),
                min_value: 10_f32, 
                max_value: 100_f32,
            },
            dt_start: date_time_now(),
            grid_cache: Default::default(),
            lines_cache: Default::default(),
            plotters_svg:  Default::default(),
        };
        // res.update_svg();
        res
    }
    
    pub fn add_series(&mut self, graphic_name: &str, second: bool, names: &[&str]) {
        for name in names {
            self.series.insert(name.to_string(), LineSeries{
                name: (*name).into(),
                graphic_name: graphic_name.into(),
                graphic_second: second,
                color: iced_native::Color::BLACK,
                points: Vec::new(),
                min_max_value: None,
            });
        };
    }
    
    pub fn series(names: &[&str]) -> Self {
        let mut graphic = Self::new();
        graphic.add_series("", false, names);
        graphic
    }
    
    pub fn set_datetime_start(&mut self, dt: DateTime) {
        self.dt_start = dt;
    }
    
    pub fn append_value(&mut self, name: &str, value: impl Into<DatePoint> ) {
        let value = value.into();
        let dt = value.date_time;
        if let Some(ref mut ser) = self.series.get_mut(name) {
            ser.points.push(value);
            self.view_port.set_end(dt);
            #[cfg(not( feature = "plotters"))] self.lines_cache.clear();
        }
    }
    

    pub fn set_values(&mut self, name: &str, values: Vec<DatePoint>) {
        if values.len()<2 {return;}
        

        let dt_start = values.first().unwrap().date_time;
        let dt_end = values.last().unwrap().date_time;

        if let Some(ser) = self.series.get_mut(name) {
            ser.points = values;
            ser.calc_min_max_value();
        }
    
        self.view_port = ViewPort {
            end: dt_end.into(),
            start: dt_start.into(),
            min_value: 0_f32,
            max_value: 100_f32,
        }
    }
    
    pub fn reset_values(&mut self) {
        for (name, s) in &mut self.series {
            s.points = Vec::new();
        }
        self.dt_start = date_time_now();
    }
    
    #[cfg(any(not(feature = "plotters"), feature = "iced_backend"))]
    pub fn view<'a>(&'a mut self) -> Element<'a, Message> {
        Canvas::new(self)
            .width(Length::Units(1800))
            .height(Length::Units(850))
        .into()
    }
    
}

// impl Drop for Graphic {
//     fn drop(&mut self) {
//         coarse_prof::write(&mut std::io::stdout()).unwrap();
//     }
// }

#[cfg(any(not(feature = "plotters"), feature = "iced_backend"))]
impl canvas::Program<Message> for Graphic {

    fn update(
        &mut self,
        _event: Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> (iced::canvas::event::Status, Option<Message>) {
        (iced::canvas::event::Status::Ignored , None)
    }
    
    #[cfg(not(feature = "plotters"))]
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
//         dbg!(&self.state.series);

        let grid = self.grid_cache.draw(bounds.size(), |frame| {
            let lines = Path::new(|p| {
                let step_x = 100;
                let h = bounds.size().height;
                for x in (1..=10).map(|x| (x*step_x) as f32) {
                    p.move_to(Point{x: x, y: 0_f32});
                    p.line_to(Point{x: x, y: h});
                }
                
                let step_y = 100;
                let w = bounds.size().width;
                for y in (1..=10).map(|y| (y*step_y) as f32) {
                    p.move_to(Point{x: 0_f32, y: y});
                    p.line_to(Point{x: w, y: y});
                }
            });
            frame.stroke(&lines, Stroke::default().with_width(1.0));
        });
        
//         dbg!(&bounds);
        let lines = self.lines_cache.draw(bounds.size(), |frame| {
            let lines = Path::new(|path| {
                for s in self.series.iter() {
                    if s.points.len() < 2 {continue;}
                    
                    let points = self.view_port.get_slice_points(&s.points);
                    let mut itr = averge_iterator(points, 200);                    
//                     let mut itr = points.iter();

                    let (x, y) = self.view_port.calc_point(&itr.next().unwrap(), bounds.size());
                    path.move_to(Point{x: x, y: y});
                    
                    for p in itr {
                        let (x, y) = self.view_port.calc_point(&p, bounds.size());
                        path.line_to(Point{x: x, y: y});
                        path.move_to(Point{x: x, y: y});
                    }
                }
            });
            frame.stroke(&lines, Stroke::default().with_width(2.0));
        });
        
        vec![grid, lines]
    }

    #[cfg(feature = "iced_backend")]
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let plot = self.lines_cache.draw(bounds.size(), |frame| {
            use plotters::prelude::*;
            let start = self.view_port.start;
            let end = self.view_port.end;
            let dlt_time_f32 = |dt: DateTime| 
                (dt - self.dt_start).to_std()
                    .and_then(|std| Ok(std.as_secs_f32()))
                    .unwrap_or(0_f32);
            let seconds_range = dlt_time_f32(start)..dlt_time_f32(end);
            if seconds_range.start >= seconds_range.end {return;}
            
            let back = iced_backend::IcedBackend::new(frame).unwrap();
            // self.update_plotters(back, seconds_range, false);
        });
        vec![plot]
    }
}

#[derive(Debug)]
struct LineSeries {
    name: String,
    graphic_name: String,
    graphic_second: bool,
    color: iced_native::Color,
    points: Vec<DatePoint>,
    min_max_value: Option<(f32, f32)>,
}

impl LineSeries {
    fn calc_min_max_value(&mut self) -> Option<(f32, f32)> {
        let min = self.points.iter()
            .min_by(|a, b| a.value.partial_cmp(&b.value).unwrap())?.value;
        let max = self.points.iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())?.value;
        self.min_max_value = Some((min, max));
        self.min_max_value
    }
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
    fn calc_point(&self, p: &DatePoint, size: Size) -> (f32, f32) {
        let y = self.calc_procent_vertical(p.value)   * size.height;
        let x = self.calc_procent_horizont(p.date_time)      * size.width;
        (x, y)
    }
    
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