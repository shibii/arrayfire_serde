extern crate arrayfire;
extern crate arrayfire_serde;
extern crate serde;
extern crate serde_test;

use arrayfire::{Array, DType, Dim4};
use serde_test::{assert_ser_tokens, Deserializer, Token};
use arrayfire_serde::{deserialize, Ser};

#[test]
fn test_dim4() {
    let dim = Dim4::new(&[1, 2, 3, 4]);
    let tokens = [
        Token::Tuple { len: 4 },
        Token::U64(1),
        Token::U64(2),
        Token::U64(3),
        Token::U64(4),
        Token::TupleEnd,
    ];
    assert_ser_tokens(&Ser::new(&dim), &tokens);

    let mut de = Deserializer::new(&tokens);
    let deserialized = deserialize::<Dim4, _>(&mut de).unwrap();
    assert_eq!(&deserialized, &dim);
    assert_eq!(de.next_token_opt(), None);
}

#[test]
fn test_dtype() {
    let dtype = DType::F64;
    let tokens = [Token::U8(2)];
    assert_ser_tokens(&Ser::new(&dtype), &tokens);

    let mut de = Deserializer::new(&tokens);
    let deserialized = deserialize::<DType, _>(&mut de).unwrap();
    assert_eq!(&deserialized, &dtype);
    assert_eq!(de.next_token_opt(), None);
}

#[test]
fn test_array() {
    let dim = Dim4::new(&[2, 2, 1, 1]);
    let values: [f64; 4] = [1.0, 2.0, 3.0, 4.0];
    let array = Array::new::<f64>(&values, dim);
    let tokens = [
        Token::Seq { len: Some(3) },
        Token::U8(2),
        Token::Tuple { len: 4 },
        Token::U64(2),
        Token::U64(2),
        Token::U64(1),
        Token::U64(1),
        Token::TupleEnd,
        Token::Seq { len: Some(4) },
        Token::F64(1.0),
        Token::F64(2.0),
        Token::F64(3.0),
        Token::F64(4.0),
        Token::SeqEnd,
        Token::SeqEnd,
    ];
    assert_ser_tokens(&Ser::new(&array), &tokens);

    let mut de = Deserializer::new(&tokens);
    let de_array = deserialize::<Array, _>(&mut de).unwrap();
    assert_eq!(array.get_type(), de_array.get_type());
    assert_eq!(array.dims(), de_array.dims());

    let mut array_vec: Vec<f64> = vec![0f64; array.elements()];
    array.host(&mut array_vec.as_mut_slice());
    let mut de_array_vec: Vec<f64> = vec![0f64; de_array.elements()];
    de_array.host(&mut de_array_vec.as_mut_slice());
    assert_eq!(array_vec, de_array_vec);
}
