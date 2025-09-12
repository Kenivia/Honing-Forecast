use honing_forecast::*;

pub fn main() {
    let (mats, prob): (Vec<i64>, f32) = chance_to_cost_test_wrapper(
        [
            [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, false, false, false, false, false, true, true, false, false, false,
                false,
            ]
            .to_vec(),
            [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, false, false, false, false, false, true, true, false, false, false,
                false,
            ]
            .to_vec(),
            [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, false, false, false, false, false, true, true, false, false, false,
                false,
            ]
            .to_vec(),
            [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, false, false, false, false, false, true, true, false, false, false,
                false,
            ]
            .to_vec(),
            [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, false, false, false, false, false, true, true, false, false, false,
                false,
            ]
            .to_vec(),
            [
                false, false, false, false, false, false, false, false, false, false, false, false,
                false, false, false, false, false, false, false, true, true, false, false, false,
                false,
            ]
            .to_vec(),
        ]
        .to_vec(),
        [
            [false, false, true, false].to_vec(),
            [false, false, true, false].to_vec(),
            [false, false, true, false].to_vec(),
            [false, false, true, false].to_vec(),
            [false, false, true, false].to_vec(),
            [false, false, true, false].to_vec(),
        ]
        .to_vec(),
        69.0,
        "Juice on grace".to_owned(),
    );
    println!("{:?}", mats);
    println!("{}", prob.to_string());

    // let my_data: Payload = serde_json::from_str(js_str).unwrap();

    // let (chance, reason): (f64, String) = cost_to_chance_test_wrapper(
    //     [
    //         [
    //             false, false, false, false, false, false, false, false, false, false, false, false,
    //             false, false, false, false, false, false, false, true, true, false, false, false,
    //             false,
    //         ]
    //         .to_vec(),
    //         [
    //             false, false, false, false, false, false, false, false, false, false, false, false,
    //             false, false, false, false, false, false, false, true, true, false, false, false,
    //             false,
    //         ]
    //         .to_vec(),
    //         [
    //             false, false, false, false, false, false, false, false, false, false, false, false,
    //             false, false, false, false, false, false, false, true, true, false, false, false,
    //             false,
    //         ]
    //         .to_vec(),
    //         [
    //             false, false, false, false, false, false, false, false, false, false, false, false,
    //             false, false, false, false, false, false, false, true, true, false, false, false,
    //             false,
    //         ]
    //         .to_vec(),
    //         [
    //             false, false, false, false, false, false, false, false, false, false, false, false,
    //             false, false, false, false, false, false, false, true, true, false, false, false,
    //             false,
    //         ]
    //         .to_vec(),
    //         [
    //             false, false, false, false, false, false, false, false, false, false, false, false,
    //             false, false, false, false, false, false, false, true, true, false, false, false,
    //             false,
    //         ]
    //         .to_vec(),
    //     ]
    //     .to_vec(),
    //     [
    //         [false, false, true, false].to_vec(),
    //         [false, false, true, false].to_vec(),
    //         [false, false, true, false].to_vec(),
    //         [false, false, true, false].to_vec(),
    //         [false, false, true, false].to_vec(),
    //         [false, false, true, false].to_vec(),
    //     ]
    //     .to_vec(),
    //     [
    //         431777, 1064398, 23748, 9010948, 15125, 1803792, 4294967295, 0, 0, 0,
    //     ]
    //     .to_vec(),
    // );
    // println!("{}", chance.to_string());
    // println!("{}", reason.to_string());
}
