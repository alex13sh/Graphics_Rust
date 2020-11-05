use iced::{
    canvas::{
        self, Cache, Canvas, Cursor, Event, Frame, Geometry, Path, Text,
    },
    mouse, Color, Element, HorizontalAlignment, Length, Point, Rectangle,
    Size, Vector, VerticalAlignment,
};

type DateTime = chrono::DateTime<chrono::Local>;
pub struct Graphic<'a> {
    state: &'a mut State
}

#[derive(Default)]
pub struct State {
    series: Vec<LineSeries>
}

impl State {
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
            series: series
        }
    }
    
    pub fn update(&mut self, message: Message) {
        match message {
        Message::AppendValues(/*dt,*/ values) => {
            for (s, v) in self.series.iter_mut().zip(values.into_iter()) {
                s.points.append_value(v);
            }
        },
        _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AppendValues(/*DateTime,*/ Vec<f32> ),
    LoadLog()
}

impl <'a> Graphic <'a> {

    pub fn new(state: &'a mut State) -> Self {
        Self {
            state: state
        }
    }
}

impl<'a> canvas::Program<Message> for Graphic <'a> {

    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<Message> {
        None
    }
    
    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        vec![]
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

#[derive(Default)]
struct LineSeries {
    name: String,
    color: iced_native::Color,
    points: Vec<DatePoint>,
}

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


// LineSeries iter into 
// impl Iterator for LineSeriesIter;
// type Item = iced::Point;
