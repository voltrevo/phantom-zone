use std::fs;

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
}
