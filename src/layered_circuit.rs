use std::collections::{HashMap, HashSet};

use bristol_circuit::BristolCircuit;

#[derive(Debug)]
pub struct LayeredCircuit {
    pub wire_count: usize,
    pub inputs: Vec<CircuitLabel>,
    pub outputs: Vec<CircuitLabel>,
    pub layers: Vec<Layer>,
}

#[derive(Debug)]
struct CircuitLabel {
    pub name: String,
    pub start: usize,
    pub bits: usize,
}

#[derive(Debug)]
struct Layer {
    pub gates: Vec<Gate>,
    pub prunes: Vec<usize>,
}

#[derive(Clone, Debug)]
enum BinaryOp {
    And,
    Or,
    Xor,
}

#[derive(Clone, Debug)]
enum UnaryOp {
    Not,
    Copy,
}

#[derive(Clone, Debug)]
enum Gate {
    Unary {
        op: UnaryOp,
        in_: usize,
        out: usize,
    },
    Binary {
        op: BinaryOp,
        a: usize,
        b: usize,
        out: usize,
    },
}

impl Gate {
    fn inputs(&self) -> Vec<usize> {
        match self {
            Self::Unary { in_, .. } => vec![*in_],
            Self::Binary { a, b, .. } => vec![*a, *b],
        }
    }

    fn out(&self) -> usize {
        match self {
            Self::Unary { out, .. } => *out,
            Self::Binary { out, .. } => *out,
        }
    }
}

impl LayeredCircuit {
    pub fn from_bristol(bristol_circuit: &BristolCircuit) -> Self {
        assert!(
            bristol_circuit.info.constants.is_empty(),
            "Bristol constants are not supported",
        );

        let inputs = io_labels(
            &bristol_circuit.info.input_name_to_wire_index,
            bristol_circuit.io_widths.0.clone(),
        );

        let outputs = io_labels(
            &bristol_circuit.info.output_name_to_wire_index,
            bristol_circuit.io_widths.1.clone(),
        );

        let input_wires = io_wires(&inputs);
        let output_wires = io_wires(&outputs);

        let gates = ingest_bristol_gates(&bristol_circuit.gates);

        Self {
            wire_count: bristol_circuit.wire_count,
            inputs,
            outputs,
            layers: separate_layers(
                &gates,
                bristol_circuit.wire_count,
                input_wires,
                output_wires,
            ),
        }
    }
}

fn separate_layers(
    gates: &Vec<Gate>,
    wire_count: usize,
    input_wires: Vec<usize>,
    output_wires: Vec<usize>,
) -> Vec<Layer> {
    let mut layers = Vec::<Layer>::new();

    // wire -> gate
    let mut input_wire_to_gates = HashMap::<usize, Vec<usize>>::new();

    let mut gate_deps_remaining = gates
        .iter()
        .map(|g| match g {
            Gate::Unary { .. } => 1,
            Gate::Binary { .. } => 2,
        })
        .collect::<Vec<_>>();

    let output_wire_set = output_wires.iter().collect::<HashSet<_>>();

    for (gate_i, gate) in gates.iter().enumerate() {
        for input in gate.inputs() {
            input_wire_to_gates.entry(input).or_default().push(gate_i);
        }
    }

    let mut gates_included = vec![false; gates.len()];

    let mut wires_resolved = input_wires;

    while wires_resolved.len() > 0 {
        let mut next_layer = Layer {
            gates: Vec::<Gate>::new(),
            prunes: Vec::<usize>::new(),
        };

        let mut next_wires_resolved = Vec::<usize>::new();

        for wire in wires_resolved {
            if let Some(gates_affected) = input_wire_to_gates.get(&wire) {
                for &gate_i in gates_affected {
                    gate_deps_remaining[gate_i] -= 1;

                    if gate_deps_remaining[gate_i] == 0 {
                        let gate = gates[gate_i].clone();

                        gates_included[gate_i] = true;
                        next_wires_resolved.push(gate.out());

                        for gate_input in gate.inputs() {
                            if output_wire_set.contains(&gate_input) {
                                continue;
                            }

                            let mut still_needed = false;

                            for other_gate in input_wire_to_gates.get(&gate_input).unwrap() {
                                if !gates_included[*other_gate] {
                                    still_needed = true;
                                    break;
                                }
                            }

                            if !still_needed {
                                next_layer.prunes.push(gate_input);
                            }
                        }

                        next_layer.gates.push(gate);
                    }
                }
            }
        }

        layers.push(next_layer);
        wires_resolved = next_wires_resolved;
    }

    assert!(gates_included.iter().all(|&b| b), "Not all gates included");

    let prune_count = layers.iter().map(|l| l.prunes.len()).sum::<usize>();

    assert!(
        prune_count == wire_count - output_wires.len(),
        "All non-output wires should have been pruned"
    );

    layers
}

fn ingest_bristol_gates(gates: &[bristol_circuit::Gate]) -> Vec<Gate> {
    gates
        .iter()
        .map(|gate| match gate.op.as_str() {
            "XOR" => Gate::Binary {
                op: BinaryOp::Xor,
                a: gate.inputs[0],
                b: gate.inputs[1],
                out: gate.outputs[0],
            },
            "AND" => Gate::Binary {
                op: BinaryOp::And,
                a: gate.inputs[0],
                b: gate.inputs[1],
                out: gate.outputs[0],
            },
            "OR" => Gate::Binary {
                op: BinaryOp::Or,
                a: gate.inputs[0],
                b: gate.inputs[1],
                out: gate.outputs[0],
            },
            "NOT" => Gate::Unary {
                op: UnaryOp::Not,
                in_: gate.inputs[0],
                out: gate.outputs[0],
            },
            "COPY" => Gate::Unary {
                op: UnaryOp::Copy,
                in_: gate.inputs[0],
                out: gate.outputs[0],
            },
            _ => panic!("Unsupported gate operation: {}", gate.op),
        })
        .collect()
}

fn io_labels(name_to_index: &HashMap<String, usize>, widths: Vec<usize>) -> Vec<CircuitLabel> {
    let mut ordered = name_to_index
        .iter()
        .map(|(name, &index)| (name.clone(), index))
        .collect::<Vec<_>>();

    ordered.sort_by_key(|(_, index)| *index);

    assert!(
        ordered.len() == widths.len(),
        "Mismatch between input count and input widths",
    );

    ordered
        .into_iter()
        .zip(widths.into_iter())
        .map(|((name, start), bits)| CircuitLabel { name, start, bits })
        .collect()
}

fn io_wires(labels: &Vec<CircuitLabel>) -> Vec<usize> {
    labels
        .iter()
        .flat_map(|label| (label.start..(label.start + label.bits)).collect::<Vec<_>>())
        .collect()
}
