#[cfg(feature = "csv")]
pub mod csv;
#[cfg(feature = "excel")]
pub mod excel;

pub mod invertor {

    #[test]
    #[cfg(feature = "csv")]
    fn config_csv_sort() {
        use crate::files::csv;
        let dir = "/home/user/.local/share/graphicmodbus/tables/save_invertor_top";
        let file_name = "22_11_2021 12_17_59.";

        let params = csv::read_values(format!("{}/{}.csv", dir, file_name)).unwrap();
        let params = crate::convert::iterator::invertor_parametrs_sort(params);
        csv::write_values(format!("{}/{}_sort.csv", dir, file_name), params).unwrap();
    }
}

mod inner {
    pub use futures::stream::{Stream, StreamExt};
    pub use std::path::{PathBuf, Path};
    pub use std::fs::File;
    pub use std::future::Future;
    pub use crate::utils::{get_file_path, DateTimeFix, date_time_to_string_name_short};
}
