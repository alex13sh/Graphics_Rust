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
        Self {
            series: Vec::new(),
            view_port: ViewPort {
                end: chrono::Local::now(),
                start: chrono::Local::now() - Duration::seconds(20*60),
                min_value: -10_f32, 
                max_value: 300_f32,
            },
            grid_cache: Default::default(),
            lines_cache: Default::default(),
            plotters_svg:  Default::default(),
        }
    }
    
    pub fn series(names: &[&str]) -> Self {
        let mut series = Vec::new();
        for name in names {
            series.push(LineSeries{
                name: (*name).into(),
                color: iced_native::Color::default(),
                points: Vec::new()
            });
        };
        Self {
            series: series, 
            .. Self::new()
        }
    }
    
    pub fn update(&mut self, message: Message) {
        match message {
        Message::AppendValues(/*dt,*/ values) => {
            self.append_values(values);
        },
        _ => {}
        }
    }
    pub fn append_value(&mut self, value: log::LogValue) {
        let hash = value.hash;
        for ser in self.series.iter_mut() {
            if ser.name == hash {
//                 dbg!(&value.date_time);
                let dt = DateTimeFix::parse_from_rfc3339(&(value.date_time+"+03:00")).unwrap().into();
                ser.points.push(DatePoint{
                    dt: dt,
                    value: value.value
                });
//                 dbg!(&dt);
                self.view_port.set_end(dt);
//                 dbg!(&self.view_port);
                if cfg!(feature = "plotters") {
//                     self.update_svg();
                } else {
                    self.lines_cache.clear();
                }
                return;
            }
        }
    }
    
    pub fn append_values(&mut self, values: Vec<f32>) {
        for (s, v) in self.series.iter_mut().zip(values.into_iter()) {
            s.points.append_value(v);
        }
//         dbg!(&self.series);
        #[cfg(feature = "plotters")] self.update_svg();
        #[cfg(not(feature = "plotters"))] self.lines_cache.clear();
        self.view_port.set_end(chrono::Local::now());
//         dbg!(&self.view_port.start);
    }
    
    #[cfg(not(feature = "plotters"))]
    pub fn view(&mut self) -> Element<Message> {
        Canvas::new(self)
            .width(Length::Units(1000))
            .height(Length::Units(1000))
        .into()
    }

    #[cfg(feature = "plotters")]
    pub fn update_svg(&mut self) {
        use plotters::prelude::*;
        
        let mut svg_text = String::new();
        {
        let root_area = SVGBackend::with_string(&mut svg_text, (800, 600)).into_drawing_area();
        root_area.fill(&WHITE).unwrap();
        let mut cc = ChartBuilder::on(&root_area)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .margin_right(20)
            .caption(
                format!("y = x^{}", 1 + 2 * 0),
                ("sans-serif", 40).into_font(),
            )
            .build_ranged(
                self.view_port.start..self.view_port.end, 
                self.view_port.min_value..self.view_port.max_value
            ).unwrap();
        cc.configure_mesh().x_labels(5).y_labels(3).draw().unwrap();
        
        for s in self.series.iter().filter(|s| s.points.len() >=2 ) {
            let points = self.view_port.get_slice_points(&s.points);
            let itr = averge_iterator(points, 200);  
            cc.draw_series(LineSeries::new(
                itr.map(|p| (p.dt, p.value)),
                &RED,
            )).unwrap()
            .label(&s.name);
    //         .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
        }
        }
//         dbg!(svg_text.len());
        self.plotters_svg = Some( svg::Handle::from_memory(svg_text));
    }
    #[cfg(feature = "plotters")]
    pub fn view(&mut self) -> Element<Message> {
        let content: Element<Message> = if let Some(handle) = self.plotters_svg.clone() {
            Svg::new(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {iced::Text::new("Not SVG").into()};
//         svg.into()
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}

impl canvas::Program<Message> for Graphic {

    fn update(
        &mut self,
        _event: Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> Option<Message> {
        None
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
    
    #[cfg(feature = "plotters")]
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let plotters = self.lines_cache.draw(bounds.size(), |_frame| {
            
            
//             Primitive::Svg {
//                 handle: svg::Handle::from_memory(self.svg_text.clone()),
//                 bounds: bounds,
//             }
        });
        vec![plotters]
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
    color: iced_native::Color,
    points: Vec<DatePoint>,
}

#[derive(Debug, Clone)]
struct DatePoint {
    dt: DateTime,
    value: f32
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
