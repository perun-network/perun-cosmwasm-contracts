// use schemars::gen::SchemaGenerator;
// use schemars::schema::*;
// use schemars::JsonSchema;

// // Does not require T: JsonSchema.
// impl<T> JsonSchema for [T; 0] {
//     no_ref_schema!();

//     fn schema_name() -> String {
//         "EmptyArray".to_owned()
//     }

//     fn json_schema(_: &mut SchemaGenerator) -> Schema {
//         SchemaObject {
//             instance_type: Some(InstanceType::Array.into()),
//             array: Some(Box::new(ArrayValidation {
//                 max_items: Some(0),
//                 ..Default::default()
//             })),
//             ..Default::default()
//         }
//         .into()
//     }
// }

// macro_rules! array_impls {
//     ($($len:tt)+) => {
//         $(
//             impl<T: JsonSchema> JsonSchema for [T; $len] {
//                 no_ref_schema!();

//                 fn schema_name() -> String {
//                     format!("Array_size_{}_of_{}", $len, T::schema_name())
//                 }

//                 fn json_schema(gen: &mut SchemaGenerator) -> Schema {
//                     SchemaObject {
//                         instance_type: Some(InstanceType::Array.into()),
//                         array: Some(Box::new(ArrayValidation {
//                             items: Some(gen.subschema_for::<T>().into()),
//                             max_items: Some($len),
//                             min_items: Some($len),
//                             ..Default::default()
//                         })),
//                         ..Default::default()
//                     }
//                     .into()
//                 }
//             }
//         )+
//     }
// }

// array_impls! {
//      1  2  3  4  5  6  7  8  9 10
//     11 12 13 14 15 16 17 18 19 20
//     21 22 23 24 25 26 27 28 29 30
//     31 32
// }