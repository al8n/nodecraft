use super::super::Id;

macro_rules! impl_number_based_id {
  ($($ty: ty), + $(,)?) => {
    $(
      impl Id for $ty {}
    )+
  };
}

impl_number_based_id!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128,);
