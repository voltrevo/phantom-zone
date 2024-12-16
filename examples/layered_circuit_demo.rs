use std::{collections::HashMap, fs};

use bristol_circuit::{BristolCircuit, CircuitInfo};
use phantom_zone::LayeredCircuit;
use serde_json::from_str;

fn main() {
    let circuit_info: CircuitInfo =
        from_str(&fs::read_to_string("tmp/circuit_info.json").unwrap()).unwrap();

    let circuit = BristolCircuit::from_info_and_bristol_string(
        &circuit_info,
        &fs::read_to_string("tmp/circuit.txt").unwrap(),
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
