#[cfg(feature = "plotters")]
impl Graphic {
    pub fn update_svg(&mut self) {
        if let Some(svg_text) = self.make_svg(self.view_port.start, self.view_port.end, false) {
            self.plotters_svg = Some( svg::Handle::from_memory(svg_text));
        }
        self.lines_cache.clear();
    }
    fn make_svg(&self, start: DateTime, end: DateTime, is_log: bool) -> Option<String> {
        use plotters::prelude::*;
        let dlt_time_f32 = |dt: DateTime|
            (dt - self.dt_start).to_std()
                .and_then(|std| Ok(std.as_secs_f32()))
                .unwrap_or(0_f32);
        let seconds_range = dlt_time_f32(start)..dlt_time_f32(end);
        if seconds_range.start >= seconds_range.end {return None;}
        let size = if is_log {
            (((seconds_range.end - seconds_range.start) as u32*10).max(800),
            1500)
        } else {(1200, 600)};

        let mut svg_text = String::new();
        {
            let back = SVGBackend::with_string(&mut svg_text, size);
            self.update_plotters(back, seconds_range, is_log);
        }
        Some(svg_text)
    }

    fn update_plotters<B, BE>(&self, back: B,
        seconds_range: core::ops::Range<f32>, is_log: bool)
        where
            BE: std::error::Error + Send + Sync,
            B: plotters::prelude::DrawingBackend<ErrorType=BE>,
        {

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
//         let seconds_range = dlt_time_f32(start)..dlt_time_f32(end);
//         dbg!(&seconds_range);


        let root_area = back.into_drawing_area();
        root_area.fill(&WHITE).unwrap();
        let (a_speed, (a_temp, a_amp)) = if is_log {
            let (a1, a2) = root_area.split_vertically(600);
            let (a2, a3) = a2.split_vertically(400);
            (a3, (a1, a2))
        } else {
            let size = root_area.dim_in_pixel();
            let (a1, a2) = root_area.split_horizontally(size.0*3/4);
            let (a2, a3) = a2.split_vertically(size.1/2);
            (a1, (a2, a3))
        };

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

        let mut cc_temp = {
            let mut cc = cc_build(&a_temp, "Температуры",
            self.view_port.min_value..self.view_port.max_value)
        .set_secondary_coord(seconds_range.clone(),
            (0.001_f32..1000.0f32).log_scale());
            cc.configure_mesh().x_labels(5).y_labels(20).draw().unwrap();
//         cc_map.insert(String::from("Температуры"), cc_temp.deref());
            cc};

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
//   //         cc_map.insert(String::from("Скорость"), cc_speed.deref());
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
            "Ток" => {
                let cc = &mut cc_amper;
                cc.draw_series(ls).unwrap()
            }, "Температуры" => {
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

        if is_log {
            let lst = vec![cc_temp.deref_mut(), cc_speed.deref_mut(), &mut cc_amper];
            for cc in lst {
                profile!("for mut cc in lst");
                cc.configure_series_labels()
                .background_style(&WHITE.mix(0.8))
                .border_style(&BLACK)
                .draw().unwrap();
            }
        }

    }

    #[cfg(not(feature = "iced_backend"))]
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
}