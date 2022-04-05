use super::sort;
use crate::U256;

/// get the median from an array of U256
pub fn median(array: &[U256]) -> U256 {
    let len = array.len();
    let array = sort(array);
    let mid: usize = len / 2;
    if len % 2 == 1 {
        array[mid]
    } else {
        (array[mid - 1] + array[mid]) / 2
    }
}

#[test]
fn ideal_median() {
    let numbers = vec![U256::from(1_i128), U256::from(2_i128), U256::from(3_i128)];
    let result = median(&numbers);
    assert_eq!(result, U256::from(2_i128));
}

#[test]
fn even_length() {
    let numbers = vec![
        U256::from(2_u128),
        U256::from(3_u128),
        U256::from(5_u128),
        U256::from(9_u128),
    ];
    let result = median(&numbers);
    assert_eq!(result, U256::from(4_i128));
}
