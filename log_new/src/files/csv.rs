use super::inner::*;
use csv::WriterBuilder;

pub fn read_values<T>(file_name: impl AsRef<Path>) -> Option<impl Iterator<Item=T>>
where T: for<'de> serde::Deserialize<'de>
{
    
    let file = File::open(file_name).ok()?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_reader(file);
        
    let itr =  std::iter::from_fn(move || {
        rdr.deserialize()
            .filter_map(|res| res.ok())
        }.next()
    );
    Some(itr)
}

pub fn write_values<T>(file_name: impl AsRef<Path>, values: impl Iterator<Item=T>) -> crate::MyResult 
where T: serde::Serialize
{
    let file = File::create(file_name)?;
    let mut wrt = csv::WriterBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_writer(file);
    
    for value in values {
        wrt.serialize(value)?;
    }
    
    Ok(())
}

pub fn write_values_async<T>(file_name: impl AsRef<Path>, values: impl Stream<Item=T> ) -> crate::MyResult<impl Future<Output=()>>
where T: serde::Serialize
{
    let file = File::create(file_name)?;
    let wrt = csv::WriterBuilder::new()
        .has_headers(true)
        .delimiter(b';')
        .from_writer(file);
    
//         while let Some(value) = values.next().await {
//             wrt.serialize(value)?;
//         }
    let f = async {
        let _ = values.fold(wrt, |mut wrt, v| async {
            wrt.serialize(v).unwrap();
            wrt
        }).await;
    };
    Ok(f)
}

#[test]
fn test_read_write_csv() {
    use crate::value::raw::*;
    use crate::value::ValueDate;
    use crate::async_channel::*;
    let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_04_08_2021__12_27_52_673792376";
    if let Some(lines) = read_values(&format!("{}.csv", file_path)) {
        
        let lines = lines.map(|v: ValueDate<ValueOld>| 
            ValueDate {
                date_time: v.date_time,
                value: Value::from(v.value),
            }
        );
        
        let (mut s1, r1) = broadcast(10);
        let r2 = r1.clone();
//             write_values("/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_04_08_2021__12_27_52_673792376_sync.csv", lines).unwrap();
        let f0 = async move {
            for v in lines {
                // dbg!(&v);
                s1.send(v).await.unwrap();
            }
        };
        let f1 = write_values_async(format!("{}_async_1.csv", file_path), r1).unwrap();
        let f2 = write_values_async(format!("{}_async_2.csv", file_path), r2).unwrap();
        
        let _ = futures::executor::block_on( futures::future::join3(f0, f1, f2) );
        
    } else {
        assert!(false);
    }
//         assert!(false);
}

fn get_raw_files_iter(dir: &str) -> impl Iterator<Item=PathBuf> {
    let path = get_file_path(dir);
    dbg!(&path);
    let paths = std::fs::read_dir(path).unwrap();
    let iter = paths.filter_map(|res| res.ok())
    .map(|dir| dir.path())
    .filter(|path|
        if let Some(ext) = path.extension() {
            ext == "csv"
        } else {false}
    );
//         res.sort_by_key(|p| p.metadata().unwrap().modified().unwrap());
    iter
}

fn file_name2date_time(file_name: &str) -> Option<DateTimeFix> {
    dbg!(file_name);
    let file_name = &format!("{} +0300", file_name);
    let res = DateTimeFix::parse_from_str(file_name, "value_%d_%m_%Y %H_%M_%S.csv %z")
    .or_else(|_|
        DateTimeFix::parse_from_str(file_name, "value_%d_%m_%Y__%H_%M_%S.csv %z")
    ).or_else(|_|
        DateTimeFix::parse_from_str(file_name, "value_%d_%m_%Y__%H_%M_%S_%.f.csv %z")
    ).or_else(|_|
        DateTimeFix::parse_from_str(file_name, "+value_%d_%m_%Y__%H_%M_%S_%.f_filter_0.1.csv %z")
    )
    // .inspect_err(|e| {dbg!(e);})
    .ok();
    dbg!(&res);
    res
}

#[test]
fn test_convert_raw_to_elk() {
    use crate::value::raw::*;
    use crate::value::ValueDate;
    let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_04_08_2021__12_27_52_673792376";
    if let Some(values) = read_values(&format!("{}.csv", file_path)) {
        let values = crate::convert::iterator::raw_to_elk(values);
        write_values(&format!("{}_elk.csv", file_path), values).unwrap();
    }
//         assert!(false);
}

#[test]
fn test_converts_raw_to_elk() {
    use crate::value::raw::*;
    use crate::value::ValueDate;
    use rayon::prelude::*;
    use rayon::iter::ParallelBridge;
    use rayon::prelude::ParallelIterator;

    let dir_elk = get_file_path("log/values/csv/");
    // for file_path in  {
    get_raw_files_iter("log/values/csv_raw/")
        // .par_bridge()

    .filter_map(|file_path| {
        Some((
            file_name2date_time(file_path.file_name().unwrap().to_str().unwrap())?,
            read_values(file_path)?
        ))
    })
    .map(|(dt, values)| (dt, crate::convert::iterator::raw_to_elk(values)))
    .map(|(dt, values)| (dt, crate::convert::iterator::value_date_shift_time(values, 3)) )
    .for_each(
        |(dt, values)| {
            write_values(dir_elk
                .join(date_time_to_string_name_short(&dt)).with_extension("csv"), 
                values).unwrap();
        }
    );
    assert!(false);
}

#[test]
fn test_convert_to_table() {
    let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_03_09_2021 11_58_30";
    if let Some(values) = read_values(&format!("{}.csv", file_path)) {
        use crate::convert::stream::*;
        let values = crate::convert::iterator::raw_to_elk(values);
        let lines = values_to_line(futures::stream::iter(values));
        let lines = values_line_to_hashmap(lines);
        futures::executor::block_on( write_values_async(format!("{}_table.csv", file_path), lines).unwrap() );
//             assert!(false);
    }
}

#[test]
fn test_values_line_diff() {
    let file_path = "/home/alex13sh/Документы/Программирование/rust_2/Graphics_Rust/log_new/test/value_03_09_2021 11_58_30";
    if let Some(values) = read_values::<crate::value::LogValueRawOld>(&format!("{}.csv", file_path)) {
        use crate::convert::stream::*;
        let lines = values_to_line(futures::stream::iter(values));
        let values = values_from_line_with_diff(lines);
        futures::executor::block_on( write_values_async(format!("{}_diff.csv", file_path), values).unwrap() );
    }
}