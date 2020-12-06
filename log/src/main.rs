use log::*;

fn main_1() -> std::io::Result<()> {
//     convert_log_file("values_14_09_2020__13_24_19_668.json", "log/", "new_log/")
    let js = open_json_file("values_14_09_2020__13_24_19_668.json");
    let hashs = js.get_all_hash();
    dbg!(hashs.len(), hashs);
    Ok(())
}

// За 11 секунд и 30-40 мб озу
fn test_speed() -> std::io::Result<()> {
    let paths = vec![
        "values_27_08_2020__13_08_30_042.json",
        "values_07_09_2020__13_02_37_096.json",
        "values_25_08_2020__13_41_06_111.json",
        "values_26_08_2020__16_26_04_840.json",
        "values_07_09_2020__16_13_35_221.json",
        "values_28_08_2020__16_57_26_959.json",
        "values_08_09_2020__14_28_27_576.json",
        "values_08_09_2020__14_28_33_906.json",
        "values_10_09_2020__15_36_13_274.json",
        "values_28_08_2020__17_06_20_523.json",
        "values_21_08_2020__17_31_00_188.json",
        "values_26_08_2020__15_48_12_214.json",
        "values_26_08_2020__16_05_51_804.json",
        "values_25_08_2020__15_15_21_933.json",
        "values_24_08_2020__19_19_10_684.json",
        "values_24_08_2020__19_03_16_045.json",
        "values_24_08_2020__18_31_00_766.json",
    ];
    for _ in 1..10 {
        for path in &paths {
            convert_log_file(path, "Log/", "test_log")?;
        }
    }
    Ok(())
}

fn main() {
    log::csv::test_read_csv_1("./log/sessions_1.csv")
        .unwrap();
}
