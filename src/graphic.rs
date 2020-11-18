use iced::{
    canvas::{
        self, Cache, Canvas, Cursor, Event, Frame, Geometry, Path, Stroke, Text,
    },
    mouse, Color, Element, HorizontalAlignment, Length, Point, Rectangle,
    Size, Vector, VerticalAlignment,
};

type DateTime = chrono::DateTime<chrono::Local>;
use chrono::Duration;
// use std::time::Duration;

pub struct Graphic {
    series: Vec<LineSeries>,
    view_port: ViewPort,
    
    grid_cache: Cache,
    lines_cache: Cache,
    
}


#[derive(Debug, Clone)]
pub enum Message {
    AppendValues(/*DateTime,*/ Vec<f32> ),
    LoadLog()
}

impl Graphic {

    pub fn new() -> Self {
        dbg!(chrono::Local::now() - Duration::seconds(20));
        Self {
            series: Vec::new(),
            view_port: ViewPort {
                end: chrono::Local::now(),
                start: chrono::Local::now() - Duration::seconds(10),
                min_value: -10_f32, 
                max_value: 20_f32,
            },
            grid_cache: Default::default(),
            lines_cache: Default::default(),
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
    pub fn append_values(&mut self, values: Vec<f32>) {
        for (s, v) in self.series.iter_mut().zip(values.into_iter()) {
            s.points.append_value(v);
        }
//         dbg!(&self.series);
        self.lines_cache.clear();
        self.view_port.set_end(chrono::Local::now());
//         dbg!(&self.view_port.start);
    }
    
    pub fn view(&mut self) -> Element<Message> {
        Canvas::new(self)
            .width(Length::Units(1000))
            .height(Length::Units(1000))
        .into()
    }
}

impl canvas::Program<Message> for Graphic {

    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<Message> {
        None
    }
    
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
                    
                    let mut itr = s.points.iter();
                    let (x, y) = self.view_port.calc_point(itr.next().unwrap(), bounds.size());
                    path.move_to(Point{x: x, y: y});
                    
                    for p in itr {
                        let (x, y) = self.view_port.calc_point(&p, bounds.size());
//                         dbg!(&x, &y);
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
    color: iced_native::Color,
    points: Vec<DatePoint>,
}

#[derive(Debug)]
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
}

// LineSeries iter into 
// impl Iterator for LineSeriesIter;
// type Item = iced::Point;
