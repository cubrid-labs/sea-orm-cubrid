use cubrid_protocol::value::Value as CubridValue;
use sea_orm::sea_query::Value as SeaValue;
use sea_orm_cubrid::{cubrid_value_to_sea, sea_value_to_cubrid};

#[test]
fn sea_to_cubrid_scalar_mappings() {
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Int(Some(7))),
        CubridValue::Int(7)
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::BigInt(Some(9))),
        CubridValue::Long(9)
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Float(Some(1.5))),
        CubridValue::Float(1.5)
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Double(Some(3.25))),
        CubridValue::Double(3.25)
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Bool(Some(true))),
        CubridValue::Bool(true)
    );
}

#[test]
fn sea_to_cubrid_null_mappings() {
    assert_eq!(sea_value_to_cubrid(&SeaValue::Int(None)), CubridValue::Null);
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::String(None)),
        CubridValue::Null
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Bytes(None)),
        CubridValue::Null
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Bool(None)),
        CubridValue::Null
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::BigInt(None)),
        CubridValue::Null
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Float(None)),
        CubridValue::Null
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Double(None)),
        CubridValue::Null
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Char(None)),
        CubridValue::Null
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::TinyInt(None)),
        CubridValue::Null
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::SmallInt(None)),
        CubridValue::Null
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::TinyUnsigned(None)),
        CubridValue::Null
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::SmallUnsigned(None)),
        CubridValue::Null
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Unsigned(None)),
        CubridValue::Null
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::BigUnsigned(None)),
        CubridValue::Null
    );
}

#[test]
fn sea_to_cubrid_text_and_bytes() {
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::String(Some(Box::new("hello".to_owned())))),
        CubridValue::String("hello".to_owned())
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Char(Some('x'))),
        CubridValue::String("x".to_owned())
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Bytes(Some(Box::new(vec![1, 2, 3])))),
        CubridValue::Bytes(vec![1, 2, 3])
    );
}

#[test]
fn sea_to_cubrid_tiny_int() {
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::TinyInt(Some(42))),
        CubridValue::Short(42)
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::TinyInt(Some(-1))),
        CubridValue::Short(-1)
    );
}

#[test]
fn sea_to_cubrid_small_int() {
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::SmallInt(Some(1000))),
        CubridValue::Short(1000)
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::SmallInt(Some(-500))),
        CubridValue::Short(-500)
    );
}

#[test]
fn sea_to_cubrid_tiny_unsigned() {
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::TinyUnsigned(Some(255))),
        CubridValue::Int(255)
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::TinyUnsigned(Some(0))),
        CubridValue::Int(0)
    );
}

#[test]
fn sea_to_cubrid_small_unsigned() {
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::SmallUnsigned(Some(60000))),
        CubridValue::Int(60000)
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::SmallUnsigned(Some(0))),
        CubridValue::Int(0)
    );
}

#[test]
fn sea_to_cubrid_unsigned_fits_i32() {
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Unsigned(Some(100))),
        CubridValue::Int(100)
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Unsigned(Some(2_147_483_647))),
        CubridValue::Int(2_147_483_647)
    );
}

#[test]
fn sea_to_cubrid_unsigned_overflows_i32() {
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::Unsigned(Some(u32::MAX))),
        CubridValue::String(u32::MAX.to_string())
    );
}

#[test]
fn sea_to_cubrid_big_unsigned_fits_i64() {
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::BigUnsigned(Some(999))),
        CubridValue::Long(999)
    );
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::BigUnsigned(Some(i64::MAX as u64))),
        CubridValue::Long(i64::MAX)
    );
}

#[test]
fn sea_to_cubrid_big_unsigned_overflows_i64() {
    assert_eq!(
        sea_value_to_cubrid(&SeaValue::BigUnsigned(Some(u64::MAX))),
        CubridValue::String(u64::MAX.to_string())
    );
}

#[test]
fn sea_to_cubrid_fallback_unknown_variant() {
    let val = SeaValue::ChronoDate(None);
    let result = sea_value_to_cubrid(&val);
    assert!(matches!(result, CubridValue::String(_)));
}

#[test]
fn cubrid_to_sea_scalar_mappings() {
    let (_, v) = cubrid_value_to_sea(&CubridValue::Int(11), "id");
    assert_eq!(v, SeaValue::Int(Some(11)));

    let (_, v) = cubrid_value_to_sea(&CubridValue::Long(99), "id");
    assert_eq!(v, SeaValue::BigInt(Some(99)));

    let (_, v) = cubrid_value_to_sea(&CubridValue::Float(4.5), "score");
    assert_eq!(v, SeaValue::Float(Some(4.5)));

    let (_, v) = cubrid_value_to_sea(&CubridValue::Double(7.5), "score");
    assert_eq!(v, SeaValue::Double(Some(7.5)));

    let (_, v) = cubrid_value_to_sea(&CubridValue::Bool(false), "active");
    assert_eq!(v, SeaValue::Bool(Some(false)));
}

#[test]
fn cubrid_to_sea_short() {
    let (name, v) = cubrid_value_to_sea(&CubridValue::Short(32767), "small_val");
    assert_eq!(name, "small_val");
    assert_eq!(v, SeaValue::SmallInt(Some(32767)));

    let (_, v) = cubrid_value_to_sea(&CubridValue::Short(-1), "neg");
    assert_eq!(v, SeaValue::SmallInt(Some(-1)));
}

#[test]
fn cubrid_to_sea_string_temporal_and_null() {
    let (name, v) = cubrid_value_to_sea(&CubridValue::Null, "nullable");
    assert_eq!(name, "nullable");
    assert_eq!(v, SeaValue::String(None));

    let (_, v) = cubrid_value_to_sea(
        &CubridValue::Date {
            year: 2025,
            month: 3,
            day: 14,
        },
        "d",
    );
    assert_eq!(v, SeaValue::String(Some(Box::new("2025-03-14".to_owned()))));

    let (_, v) = cubrid_value_to_sea(
        &CubridValue::Timestamp {
            year: 2025,
            month: 3,
            day: 14,
            hour: 1,
            minute: 2,
            second: 3,
        },
        "ts",
    );
    assert_eq!(
        v,
        SeaValue::String(Some(Box::new("2025-03-14 01:02:03".to_owned())))
    );

    let (_, v) = cubrid_value_to_sea(
        &CubridValue::Datetime {
            year: 2025,
            month: 3,
            day: 14,
            hour: 1,
            minute: 2,
            second: 3,
            ms: 444,
        },
        "dt",
    );
    assert_eq!(
        v,
        SeaValue::String(Some(Box::new("2025-03-14 01:02:03.444".to_owned())))
    );
}

#[test]
fn cubrid_to_sea_time() {
    let (name, v) = cubrid_value_to_sea(
        &CubridValue::Time {
            hour: 14,
            minute: 30,
            second: 59,
        },
        "t",
    );
    assert_eq!(name, "t");
    assert_eq!(v, SeaValue::String(Some(Box::new("14:30:59".to_owned()))));
}

#[test]
fn cubrid_to_sea_bytes_and_string() {
    let (_, v) = cubrid_value_to_sea(&CubridValue::String("abc".to_owned()), "name");
    assert_eq!(v, SeaValue::String(Some(Box::new("abc".to_owned()))));

    let (_, v) = cubrid_value_to_sea(&CubridValue::Bytes(vec![9, 8]), "payload");
    assert_eq!(v, SeaValue::Bytes(Some(Box::new(vec![9, 8]))));
}

#[test]
fn cubrid_to_sea_preserves_column_name() {
    let (name, _) = cubrid_value_to_sea(&CubridValue::Int(1), "my_column");
    assert_eq!(name, "my_column");

    let (name, _) = cubrid_value_to_sea(&CubridValue::Null, "");
    assert_eq!(name, "");
}
