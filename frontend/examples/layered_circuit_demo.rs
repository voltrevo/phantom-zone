use std::collections::HashMap;

use bristol_circuit::{BristolCircuit, CircuitInfo};
use frontend::LayeredCircuit;
use serde_json::from_str;

const CIRCUIT_STR: &str = include_str!("./circuits/gt10/circuit.txt");
const CIRCUIT_INFO_STR: &str = include_str!("./circuits/gt10/circuit_info.json");

fn main() {
    let circuit = BristolCircuit::from_info_and_bristol_string(
        &from_str::<CircuitInfo>(CIRCUIT_INFO_STR).unwrap(),
        CIRCUIT_STR,
    )
    .unwrap();

    let layered_circuit = LayeredCircuit::from_bristol(&circuit);

    println!("{:?}", layered_circuit);

    // this input is written for use with the greaterThan10.ts circuit (--boolify-width 4),
    // here x is 12 (8+4+0+0), which is gt 10, so the output is 10 (8+0+2+0)
    let inputs = [("x".to_string(), vec![true, true, false, false])]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();

    let outputs = layered_circuit.eval(inputs);

    println!("{:?}", outputs);
}
