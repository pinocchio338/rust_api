use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, ext_contract, PromiseResult};
use api3_common::Bytes32;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct CrossContractCallDemo {
    target: &str
}

#[near_bindgen]
impl CrossContractCallDemo {
    pub fn hello_world(&self) -> String { String::from("hello-world") }

    pub fn get_datapoint(&mut self, datapoint_id: Bytes32) {
        ext_ft::read_with_data_point_id(
            datapoint_id,
            &"test-api3.testnet", // contract account id
            0, // yocto NEAR to attach
            5_000_000_000_000 // gas to attach
        )
            .then(ext_self::my_callback(
                &near_sdk::env::current_account_id(), // this contract's account id
                0, // yocto NEAR to attach to the callback
                5_000_000_000_000 // gas to attach to the callback
            ));
    }

    pub fn my_callback(&self) -> (Bytes32, u32) {
        assert_eq!(
            near_sdk::env::promise_results_count(),
            1,
            "This is a callback method"
        );

        // handle the result from the cross contract call this method is a callback for
        match near_sdk::env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => panic!("call read_with_data_point_id failed"),
            PromiseResult::Successful(result) => {
                let r = near_sdk::serde_json::from_slice::<(Bytes32, u32)>(&result).unwrap();
                near_sdk::env::log(
                    format!("obtained result: {:?}", r).as_ref()
                );
                r
            },
        }
    }
}

#[ext_contract(ext_ft)]
trait ContractB {
    fn read_with_data_point_id(&self, data_point_id: Bytes32) -> (Bytes32, u32);
}

#[ext_contract(ext_self)]
trait MyContract {
    fn my_callback(&self) -> Bytes32;
}