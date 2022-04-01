
use umya_spreadsheet::structs::{
    Worksheet,
    Cell,
    Chart,
};

pub trait SheetInner {
    fn get_cell_by_column_and_row_mut(&mut self, col:usize, row:usize)-> &mut Cell;
    fn calculation_auto_width(&mut self);

    fn get_column_by_name(&self, name: &str) -> Option<u32>;
    fn make_coordinates_columns(&self, columns: &[&str], rows: u32) -> Vec<(String, String)>;
    fn new_chart_liner(&mut self, from: &str, to: &str, area_time: &str, names: Vec<&str>, area: Vec<&str>);
}

impl SheetInner for Worksheet {
    fn get_cell_by_column_and_row_mut(&mut self, col:usize, row:usize)-> &mut Cell {
        self.get_cell_by_column_and_row_mut(col as u32, row as u32)
    }
    fn calculation_auto_width(&mut self) {
        self.calculation_auto_width();
    }

    fn get_column_by_name(&self, name: &str) -> Option<u32> {
        for col in 1.. {
            let v = self.get_value_by_column_and_row(col, 1);
            if v.is_empty() {
                return None;
            } else if v.as_str() == name {
                return Some(col);
            }
        }
        None
    }

    fn make_coordinates_columns(&self, columns: &[&str], rows: u32) -> Vec<(String, String)> {
        let title = self.get_title();
        let rows = rows + 1;
        columns.into_iter().map(|name| {
            let column = self.get_column_by_name(name).unwrap();
            let column_char = ('A' as u8 + column as u8 - 1) as char;
            (
                format!("{title}!${column}$1", title=title, column = column_char),
                format!("{title}!${column}$2:${column}${rows}", title=title, column = column_char, rows = rows)
            )
        }).collect()
    }
    fn new_chart_liner(&mut self, from: &str, to: &str, area_time: &str, names: Vec<&str>, area: Vec<&str>) {
        let mut from_marker = umya_spreadsheet::structs::drawing::spreadsheet::MarkerType::default();
        let mut to_marker = umya_spreadsheet::structs::drawing::spreadsheet::MarkerType::default();
        from_marker.set_coordinate(from);
        to_marker.set_coordinate(to);
        
        let mut chart = umya_spreadsheet::structs::Chart::default();
        chart.new_chart(
            umya_spreadsheet::structs::ChartType::LineChart,
            from_marker,
            to_marker,
            area,
        );
        chart_axis_x(&mut chart, area_time);
        chart_series(&mut chart, names);
        chart_style(&mut chart);
        chart_axis_2(&mut chart);
        self.get_worksheet_drawing_mut().add_chart_collection(chart);
    }
}
impl SheetInner for &mut Worksheet {
    fn get_cell_by_column_and_row_mut(&mut self, col:usize, row:usize)-> &mut Cell {
        <Worksheet as SheetInner>::get_cell_by_column_and_row_mut(self, col, row)
    }
    fn calculation_auto_width(&mut self) {
        <Worksheet as SheetInner>::calculation_auto_width(self);
    }
    fn get_column_by_name(&self, name: &str) -> Option<u32> {
        <Worksheet as SheetInner>::get_column_by_name(self, name)
    }
    fn make_coordinates_columns(&self, columns: &[&str], rows: u32) -> Vec<(String, String)> {
        <Worksheet as SheetInner>::make_coordinates_columns(self, columns, rows)
    }
    fn new_chart_liner(&mut self, from: &str, to: &str, area_time: &str, names: Vec<&str>, area: Vec<&str>) {
        <Worksheet as SheetInner>::new_chart_liner(self, from, to, area_time, names, area);
    }
}

fn chart_style(chart: &mut Chart) {
    let graph = chart.get_two_cell_anchor_mut().get_graphic_frame_mut().as_mut().unwrap()
        .get_graphic_mut().get_graphic_data_mut().get_chart_space_mut();
    let chart = graph.get_chart_mut().get_plot_area_mut().get_line_chart_mut().as_mut().unwrap();
    chart.get_show_marker_mut().set_val(false);
    chart.get_grouping_mut().set_val(umya_spreadsheet::structs::drawing::charts::GroupingValues::Standard);
    // let shape = chart.get_area_chart_series_list_mut().get_area_chart_series_mut()[0].get_shape_properties_mut().as_mut().unwrap();
    for series in chart.get_area_chart_series_list_mut().get_area_chart_series_mut() {
        let marker = umya_spreadsheet::structs::drawing::charts::Marker::default();
    // dbg!(&marker);
        series.set_marker(marker);;
    }

    // 
    // dbg!(chart.get_area_chart_series_list());
}

fn chart_axis_2(chart: &mut Chart) {
    let mut ser_axis = umya_spreadsheet::structs::drawing::charts::SeriesAxis::default();
    let axis_id = ser_axis.get_axis_id().clone();
    let pos = umya_spreadsheet::structs::drawing::charts::AxisPositionValues::Right;
    ser_axis.get_axis_position_mut().set_val(pos);


    let graph = chart.get_two_cell_anchor_mut().get_graphic_frame_mut().as_mut().unwrap()
        .get_graphic_mut().get_graphic_data_mut().get_chart_space_mut();
    let plot_area = graph.get_chart_mut().get_plot_area_mut();
    plot_area.add_series_axis(ser_axis);
    let chart = plot_area.get_line_chart_mut().as_mut().unwrap();
    chart.add_axis_id(axis_id);
}

fn chart_series(chart: &mut Chart, names: Vec<&str>) {
    let graph = chart.get_two_cell_anchor_mut().get_graphic_frame_mut().as_mut().unwrap()
        .get_graphic_mut().get_graphic_data_mut().get_chart_space_mut();
    let chart = graph.get_chart_mut().get_plot_area_mut().get_line_chart_mut().as_mut().unwrap();
    let series_list = chart.get_area_chart_series_list_mut().get_area_chart_series_mut();    

    for (series, name) in series_list.into_iter().zip(names.into_iter()) {
        // series.get_formula_mut().
        
    }
}  

fn chart_axis_x(chart: &mut Chart, area_time: &str) {
    let graph = chart.get_two_cell_anchor_mut().get_graphic_frame_mut().as_mut().unwrap()
    .get_graphic_mut().get_graphic_data_mut().get_chart_space_mut();
    let chart = graph.get_chart_mut().get_plot_area_mut().get_line_chart_mut().as_mut().unwrap();
    let series_list = chart.get_area_chart_series_list_mut().get_area_chart_series_mut();

    let mut category = umya_spreadsheet::structs::drawing::charts::CategoryAxisData::default();
    let string = category.get_string_reference_mut().get_formula_mut();
    string.set_address_str(area_time);
    series_list.get_mut(0).unwrap().set_category_axis_data(category);
}