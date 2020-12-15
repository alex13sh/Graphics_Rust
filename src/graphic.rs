#![allow(dead_code, unused_imports)]

use iced::{
    canvas::{
        self, Cache, Canvas, Cursor, Event, Frame, Geometry, Path, Stroke,
    },
    Svg, svg, Container,
    mouse, Color, Element, HorizontalAlignment, Length, Point, Rectangle,
    Size, Vector, VerticalAlignment,
};

type DateTime = chrono::DateTime<chrono::Local>;
type DateTimeFix = chrono::DateTime<chrono::FixedOffset>; 
use chrono::Duration;
// use std::time::Duration;

pub struct Graphic {
    series: Vec<LineSeries>, // HashMap
    view_port: ViewPort,
    dt_start: DateTime,
    
    grid_cache: Cache,
    lines_cache: Cache,
    plotters_svg: Option<svg::Handle>,
}

#[derive(Debug, Clone)]
pub enum Message {
//     AppendValues(log::LogValue ),
    AppendValues(Vec<f32>),
    LoadLog()
}

impl Graphic {

    pub fn new() -> Self {
        let mut res = Self {
            series: Vec::new(),
            view_port: ViewPort {
                end: chrono::Local::now(),
                start: chrono::Local::now() - Duration::seconds(2*60),
                min_value: -10_f32, 
                max_value: 100_f32,
            },
            dt_start: chrono::Local::now(),
            grid_cache: Default::default(),
            lines_cache: Default::default(),
            plotters_svg:  Default::default(),
        };
        res.update_svg();
        res
    }
    
    pub fn add_series(&mut self, graphic_name: &str, second: bool, names: &[&str]) {
        for name in names {
            self.series.push(LineSeries{
                name: (*name).into(),
                graphic_name: graphic_name.into(),
                graphic_second: second,
                color: iced_native::Color::default(),
                points: Vec::new()
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
    
    pub fn update(&mut self, message: Message) {
        match message {
        Message::AppendValues(/*dt,*/ values) => {
//             self.append_values(values);
        },
        _ => {}
        }
    }
    pub fn append_log_value(&mut self, value: log::LogValue) {
        let hash = value.hash;
        let dt: DateTime  = value.date_time.into();
        let dp = DatePoint{ dt: dt, value: value.value };
        self.append_value(&hash, dp);
    }
    
    pub fn append_value(&mut self, name: &str, value: impl Into<DatePoint> ) {
        let value = value.into();
        let dt = value.dt;
        if let Some(ref mut ser) = self.series.iter_mut().find(|ser| ser.name == name) {
            ser.points.push(value);
            self.view_port.set_end(dt);
            #[cfg(not( feature = "plotters"))] self.lines_cache.clear();
        }
    }
    
    pub fn append_values(&mut self, values: Vec<(&str, f32)>) {
        let dt = chrono::Local::now();
        for (name, value) in values {
            if let Some(ref mut ser) = self.series.iter_mut().find(|ser| name.contains(&ser.name)) {
                ser.points.push(DatePoint{dt: dt, value: value});
            }
        }
        
//         #[cfg(      feature = "plotters")] self.update_svg();
        #[cfg(not( feature = "plotters"))] self.lines_cache.clear();
        self.view_port.set_end(dt);
//         dbg!(&self.view_port.start);
    }
    
    pub fn reset_values(&mut self) {
        for mut s in &mut self.series {
            s.points = Vec::new();
        }
    }
    
    #[cfg(not(feature = "plotters"))]
    pub fn view<'a>(&mut self) -> Element<'a, Message> {
        Canvas::new(self)
            .width(Length::Units(1000))
            .height(Length::Units(1000))
        .into()
    }

    #[cfg(feature = "plotters")]
    pub fn update_svg(&mut self) {
        if let Some(svg_text) = self.make_svg(self.view_port.start, self.view_port.end, false) {
            self.plotters_svg = Some( svg::Handle::from_memory(svg_text));
        }
    }
    
    #[cfg(feature = "plotters")]
    fn make_svg(&self, start: DateTime, end: DateTime, is_log: bool) -> Option<String> {
        use coarse_prof::profile;
        profile!("update_svg");
        
        use plotters::prelude::*;
        use std::collections::HashMap;
        use std::ops::{Deref, DerefMut};
        
//         let dt_range = self.view_port.start..self.view_port.end;
        let dlt_time_f32 = |dt: DateTime| 
            if let Ok(std) = (dt - self.dt_start).to_std() {
                std.as_secs_f32()
            } else {
                0_f32
            };
        let seconds_range = dlt_time_f32(start)..dlt_time_f32(end);
//         dbg!(&seconds_range);
        if seconds_range.start >= seconds_range.end {return None;}
        
        let mut svg_text = String::new();
        {
        profile!("svg_text");
        let size = if is_log {
            (((seconds_range.end - seconds_range.start) as u32*10).max(800),
            1000)
        } else {(1200, 600)};
        let root_area = SVGBackend::with_string(&mut svg_text, size).into_drawing_area();
        root_area.fill(&WHITE).unwrap();
        let (a_speed, a_temp) = if is_log {
            let (upper, lower) = root_area.split_vertically(400);
            (lower, upper)
        } else {root_area.split_horizontally(900)};
        
        let cc_build = |on, graphic_name, range_1| {
            ChartBuilder::on(on)
            .x_label_area_size(25)
            .y_label_area_size(40)
            .right_y_label_area_size(40)
            .margin(5)
//             .margin_right(20)
            .caption(
                graphic_name, // date name
                ("sans-serif", 20).into_font(),
            ).build_ranged(
                seconds_range.clone(), 
                range_1
            ).unwrap()
            };
        
//         let mut cc_map = HashMap::new();
        
        let mut cc_temp = cc_build(&a_temp, "Температуры",
            self.view_port.min_value..self.view_port.max_value)
        .set_secondary_coord(seconds_range.clone(),
            (0.001_f32..1000.0f32).log_scale());
        cc_temp.configure_mesh().x_labels(5).y_labels(10).draw().unwrap();
//         cc_map.insert(String::from("Температуры"), cc_temp.deref());
        
        let mut cc_speed = cc_build(&a_speed, "Скорость",
            0_f32..25_000_f32)
            .set_secondary_coord(seconds_range.clone(),
            0_f32..25_f32);
            cc_speed.configure_mesh()
                .x_labels(20).y_labels(8)
                .y_desc("Скорость (об./c)")
                .y_label_formatter(&|x| format!("{}", *x as u32))
                .draw().unwrap();
            cc_speed.configure_secondary_axes()
                .x_labels(20).y_labels(10)
                .y_desc("Вибрация (м/с^2)")
                .y_label_formatter(&|x| format!("{:2.}", x))
                .draw().unwrap();
//         cc_map.insert(String::from("Скорость"), cc_speed.deref());
//         let color = Palette99::pick(idx).mix(0.9);
        for (s, c) in self.series.iter().filter(|s| s.points.len() >=2 ).zip(0..) {
            profile!("self.series.iter()");
            let points = if is_log {
                &s.points
            } else {
                self.view_port.get_slice_points(&s.points)
            };
//             let itr = averge_iterator(points, 200);
            let itr = points.iter();
            let ls = LineSeries::new(
                itr.map(|p| (dlt_time_f32(p.dt), p.value)),
                &Palette99::pick(c),
            );
            let ser = match s.graphic_name.deref() {
            "Температуры" => {
                let cc = &mut cc_temp;
                if s.graphic_second {
                    cc.draw_secondary_series(ls)
                } else {
                    cc.draw_series(ls)
                }.unwrap()
            }, "Скорость" | _ => {
                let cc = &mut cc_speed;
                if s.graphic_second {
                    cc.draw_secondary_series(ls)
                } else {
                    cc.draw_series(ls)
                }.unwrap()
            },
            };
            ser
            .label(&s.name)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &Palette99::pick(c)));
        }
        
//         let lst = vec![cc_temp.deref_mut(), cc_speed.deref_mut()];
//         for mut cc in lst {
//             profile!("for mut cc in lst");
//             cc.configure_series_labels()
//             .background_style(&WHITE.mix(0.8))
//             .border_style(&BLACK)
//             .draw().unwrap();
//         }
        }
        Some(svg_text)
    }
    #[cfg(feature = "plotters")]
    pub fn view<'a>(&mut self) -> Element<'a, Message> {
        use coarse_prof::profile;
        profile!("Graphic view");
        let content: Element<Message> = if let Some(handle) = self.plotters_svg.clone() {
            Svg::new(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {iced::Text::new("Not SVG").into()};
        content.into()
//         Container::new(content)
//             .width(Length::Fill)
//             .height(Length::Fill)
//             .padding(20)
// //             .center_x()
// //             .center_y()
//             .into()
    }
    
    pub fn save_svg(&self) {
        if let Some(svg_text) = self.make_svg(self.dt_start, self.view_port.end, true) {
            use std::io::Write;
            let mut f = std::fs::File::create(format!("./plot/plot_{}.svg", log::date_time_to_string_name(&self.dt_start.into()))).unwrap();
            f.write(svg_text.as_bytes());
            f.flush();
        }
    }
}

impl Drop for Graphic {
    fn drop(&mut self) {
        coarse_prof::write(&mut std::io::stdout()).unwrap();
    }
}

#[cfg(not(feature = "plotters"))]
impl canvas::Program<Message> for Graphic {

    fn update(
        &mut self,
        _event: Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> Option<Message> {
        None
    }
    
//     #[cfg(not(feature = "plotters"))]
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
    
}

// struct Legend {
//     // color, visible, 
// }
// 
// struct AbstractAxis {
//     visible: bool,
//     lineVisible: bool,
//     color: Color
// }
// 
// struct ValueAxis {
//     values: Vec<f32>
// }

#[derive(Default, Debug)]
struct LineSeries {
    name: String,
    graphic_name: String,
    graphic_second: bool,
    color: iced_native::Color,
    points: Vec<DatePoint>,
}

#[derive(Debug, Clone)]
pub struct DatePoint {
    dt: DateTime,
    value: f32
}

impl DatePoint {
    pub fn from_value(value: f32) -> Self {
        Self {
            dt: chrono::Local::now(),
            value: value,
        }
    }
}

impl From<f32> for DatePoint {
    fn from(value: f32) -> DatePoint {
        Self {
            dt: chrono::Local::now(),
            value: value,
        }
    }
}

trait VecDatePoint {
    fn append_value(&mut self, value: f32);
}
impl VecDatePoint for Vec<DatePoint> {
    fn append_value(&mut self, value: f32) {
        self.push(
            DatePoint {
                dt: chrono::Local::now(),
                value: value
            }
        );
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
        let x = self.calc_procent_horizont(p.dt)      * size.width;
        (x, y)
    }
    
    fn set_end(&mut self, end: DateTime) {
        let dlt = self.end - self.start;
        self.end = end;
        self.start = self.end - dlt;
    }
    
    fn get_slice_points<'a>(&self, points: &'a Vec<DatePoint>) -> &'a [DatePoint] {
        let points = &points[..];
        let i_start=match points.binary_search_by(|point| point.dt.cmp(&self.start)) {Ok(pos)=>pos, Err(pos)=>pos};
        let i_end=match points.binary_search_by(|point| point.dt.cmp(&self.end)) {Ok(pos)=>pos, Err(pos)=>pos};
        &points[i_start..i_end]
    }
}

// LineSeries iter into 
// impl Iterator for LineSeriesIter;
// type Item = iced::Point;

fn averge_iterator(points: &[DatePoint], max_points: usize) -> impl Iterator<Item=DatePoint> + '_ {
    points.chunks(points.len()/max_points+1).map(|points| {
        let sum_value = points.iter().fold(0_f32, |value, point| value + point.value);
        DatePoint{
            dt: points.first().unwrap().dt ,
            value: sum_value / points.len() as f32
        }
    })
}
