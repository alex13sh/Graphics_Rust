use super::*;
#[cfg(test)]
fn test_value_init() -> Value {
    Value::from(ValueInit{
        name: "Name_1".into(),
        suffix_name: Some("".into()),
        address: 1,
        direct: ValueDirect::Write,
        size: ValueSize::FLOAT,
        log: None,
    })
}

// try into bool, i32, f32

// #[test]
// fn test_value_ops_bit() {
//     let v = Value::from(ValueInit{
//         name: "Name_1".into(),
//         address: 1,
//         direct: ValueDirect::Write,
//         size: ValueSize::BitMap(vec![]),
//         log: None,
//     });
//     v.set_bit(1, true);
//     assert_eq!(v.value.get(), 2);
//     v.set_bit(4, true);
//     assert_eq!(v.value.get(), 18);
//     assert_eq!(v.get_bit(3), false);
//     assert_eq!(v.get_bit(4), true);
// }

#[test]
fn test_value_into_f32() {
    let v = test_value_init();
    *v.value.lock().unwrap() = (u32::from_le_bytes([0x00,0x00,0x20,0x3E]), false);
    let f: f32 = (&v).try_into().unwrap();
    assert_eq!(f, 0.15625);
    let f = f32::from_le_bytes([0x00,0x00,0x20,0x3E]);
    assert_eq!(f, 0.15625);
}

