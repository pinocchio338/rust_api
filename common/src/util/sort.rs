use crate::U256;

/// sort an array of U256
pub fn sort(array: &[U256]) -> Vec<U256> {
    let mut array = array.to_vec();
    array.sort();
    array
}

#[test]
fn already_sorted() {
    let numbers = vec![U256::from(1_i128), U256::from(2_i128), U256::from(3_i128)];
    let result = sort(&numbers);
    assert_eq!(result, numbers);
}

#[test]
fn unsorted() {
    let numbers = vec![U256::from(2_i128), U256::from(1_i128), U256::from(3_i128)];
    let result = sort(&numbers);
    let expected = vec![U256::from(1_i128), U256::from(2i128), U256::from(3_i128)];
    assert_eq!(result, expected);
}

#[test]
fn large_numbers() {
    let numbers = vec![
        U256::from(212837128371931812_u128),
        U256::from(u128::MAX),
        U256::from(51623219381273_u128),
    ];
    let result = sort(&numbers);
    let expected = vec![
        U256::from(51623219381273_u128),
        U256::from(212837128371931812_u128),
        U256::from(u128::MAX),
    ];
    assert_eq!(result, expected);
}

#[test]
fn max_numbers() {
    let numbers = vec![U256([u64::MAX; 4]), U256::from([u8::MAX; 32])];
    let result = sort(&numbers);
    let expected = vec![U256::from([u8::MAX; 32]), U256([u64::MAX; 4])];
    assert_eq!(result, expected);
}
