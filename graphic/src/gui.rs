use iced::{
    canvas::{
        self, Cache, Canvas, Cursor, Event, Frame, Geometry, Path, Stroke,
    },
    Svg, svg, Container,
    mouse, Color, Element, HorizontalAlignment, Length, Point, Rectangle,
    Size, Vector, VerticalAlignment,
};

use crate::{
    LineSeries,
    ViewPort,
    DateTime,
    date_time_now,
    HashMap,
};

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
                // graphic_name: graphic_name.into(),
                graphic_second: second,
                // color: iced_native::Color::BLACK,
                points: Vec::new(),
                // min_max_value: None,
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
            // ser.calc_min_max_value();
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
    pub fn view(&mut self) -> Element<Message> {
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

impl Graphic {
    /*
    fn update_plotters<B, BE>(&self, back: B,
        seconds_range: core::ops::Range<f32>, is_log: bool)
        where
            BE: std::error::Error + Send + Sync,
            B: plotters::prelude::DrawingBackend<ErrorType=BE>,
    {
        use plotters::prelude::*;
        let root_area = back.into_drawing_area();
        root_area.fill(&WHITE).unwrap();

        let cc_build = |on, graphic_name: &str, range_1| {
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

        let mut cc_speed = {
            let mut cc = cc_build(&a_speed, "Скорость",
            0_f32..25_000_f32)
            .set_secondary_coord(seconds_range.clone(),
            0_f32..25_f32);
            cc.configure_mesh()
                .x_labels(20).y_labels(8)
                .y_desc("Скорость (об./м)")
                .y_label_formatter(&|x| format!("{}", *x as u32))
                .draw().unwrap();
            cc.configure_secondary_axes()
                .x_labels(20).y_labels(10)
                .y_desc("Вибрация (м/с^2)")
                .y_label_formatter(&|x| format!("{:2.}", x))
                .draw().unwrap();
                cc};

        let mut cc_amper = {
            let mut cc = cc_build(&a_amp, "Ток",
            0_f32..120_f32);
//             .set_secondary_coord(seconds_range.clone(), 0_f32..25_f32);
            cc.configure_mesh()
                .x_labels(5).y_labels(12)
//                 .y_desc("Ток (об./м)")
                .y_label_formatter(&|x| format!("{}", *x as u32))
                .draw().unwrap();
            cc};


    } */
}