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

#[derive(Debug, Clone)]
pub enum Message {
    AppendValues(DateTime, Vec<f32> ),
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

struct LineSeries {
    points: Vec<DatePoint>,
    color: iced_native::Color
}

struct DatePoint {
    dt: DateTime,
    value: f32
}

// LineSeries iter into 
// impl Iterator for LineSeriesIter;
// type Item = iced::Point;
