use discriminant::{Discriminant, Enum};

#[derive(Enum)]
#[allow(unused)]
#[repr(u8)]
pub enum MixedEnum<T> {
    UnitVariantA = 1,
    TupleVariantB(i32, f64),
    StructVariantC { name: String, value: T },
    SomeValue(T),
    NoneValue = 99,
    TupleWithGeneric(T, usize),
}

#[test]
fn test() {
    assert_eq!(
        <MixedEnum::<()> as Enum>::Discriminant::all().collect::<Vec<_>>(),
        vec![
            <MixedEnum::<()> as Enum>::Discriminant::UnitVariantA,
            <MixedEnum::<()> as Enum>::Discriminant::TupleVariantB,
            <MixedEnum::<()> as Enum>::Discriminant::StructVariantC,
            <MixedEnum::<()> as Enum>::Discriminant::SomeValue,
            <MixedEnum::<()> as Enum>::Discriminant::NoneValue,
            <MixedEnum::<()> as Enum>::Discriminant::TupleWithGeneric,
        ]
    );
    assert_eq!(
        std::convert::identity::<u8>(MixedEnum::<()>::UnitVariantA.discriminant().into()),
        1
    );
    assert_eq!(
        std::convert::identity::<u8>(MixedEnum::<()>::SomeValue(()).discriminant().into()),
        4
    );
}
