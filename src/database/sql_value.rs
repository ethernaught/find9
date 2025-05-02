pub enum SqlValue {
    Int(i128),
    Uint(u128),
    Float(f64),
    Str(String)
}

macro_rules! impl_from_signed {
    ($($t:ty),*) => {
        $(
            impl From<$t> for SqlValue {

                fn from(value: $t) -> Self {
                    SqlValue::Uint(value as u128)
                }
            }
        )*
    };
}

impl_from_signed!(i8, i16, i32, i64, i128, isize);

macro_rules! impl_from_unsigned {
    ($($t:ty),*) => {
        $(
            impl From<$t> for SqlValue {

                fn from(value: $t) -> Self {
                    SqlValue::Uint(value as u128)
                }
            }
        )*
    };
}

impl_from_unsigned!(u8, u16, u32, u64, u128, usize);

macro_rules! impl_from_float {
    ($($t:ty),*) => {
        $(
            impl From<$t> for SqlValue {

                fn from(value: $t) -> Self {
                    SqlValue::Float(value as f64)
                }
            }
        )*
    };
}

impl_from_float!(f32, f64);

impl From<bool> for SqlValue {

    fn from(value: bool) -> Self {
        SqlValue::Str(value.to_string())
    }
}

impl From<&str> for SqlValue {

    fn from(value: &str) -> Self {
        SqlValue::Str(value.to_string())
    }
}

impl From<String> for SqlValue {

    fn from(value: String) -> Self {
        SqlValue::Str(value)
    }
}
