use log_new as log;

#[tokio::main]
async fn main() {
    #[cfg(feature = "file")]
    log::test::convert_csv_to_excel_2().await;
}