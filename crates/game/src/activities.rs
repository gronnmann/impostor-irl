use serde_json::{Value, json};

pub const KINDS: [&str; 17] = [
    "wires",
    "code",
    "signal",
    "battery",
    "memory",
    "anomaly",
    "circuits",
    "password",
    "mixing",
    "transfer",
    "fault",
    "frequency",
    "breaker",
    "timing",
    "route_power",
    "switches",
    "cargo",
];
#[derive(Clone, Debug, serde::Serialize)]
pub struct Station {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub description: String,
    pub duration: String,
    pub location: String,
}
pub fn stations() -> Vec<Station> {
    let metadata = [
        ("Connect Wires", "Connect matching terminals."),
        ("Enter Code", "Remember and enter the code."),
        ("Calibrate Signal", "Hold the target frequency."),
        ("Charge Battery", "Stop inside the charge zone."),
        ("Pattern Memory", "Repeat the flashing sequence."),
        ("Find Anomaly", "Find the different symbol."),
        (
            "Align Circuits",
            "Connect the west inlet to the east outlet.",
        ),
        ("Crack Password", "Solve three clues for the PIN."),
        ("Chemical Mixing", "Balance and hold the chemical mixture."),
        ("Data Transfer", "Keep the cursor inside the moving target."),
        ("Find the Fault", "Find the light with irregular timing."),
        ("Frequency Hunt", "Tune the strongest signal and hold it."),
        ("Code Breaker", "Use digit feedback to discover the code."),
        ("Timing Challenge", "Stop in the narrow zone three times."),
        (
            "Route the Power",
            "Rotate pipes to connect the power route.",
        ),
        (
            "Switch Sequence",
            "Activate the switches in the shown order.",
        ),
        ("Cargo Sorting", "Sort each cargo item into its category."),
    ];
    KINDS
        .iter()
        .zip(metadata)
        .map(|(kind, (name, description))| Station {
            id: (*kind).into(),
            kind: (*kind).into(),
            name: name.into(),
            description: description.into(),
            duration: "10–30 sec".into(),
            location: String::new(),
        })
        .collect()
}
pub fn challenge(kind: &str, seed: u64) -> Value {
    let n = seed as usize;
    match kind {
        "wires" => {
            let mut order = vec![0, 1, 2, 3];
            order.rotate_left(n % 4);
            json!({"colors":["cyan","red","amber","violet"],"right_order":order})
        }
        "code" => json!({"code":format!("{:05}",10000+seed%90000),"reveal_ms":2500}),
        "signal" => json!({"target":30+seed%41,"tolerance":3,"hold_ms":1500}),
        "battery" => json!({"zone_start":35+seed%10,"zone_end":55+seed%10,"required_hits":3}),
        "memory" => {
            json!({"sequence":(0..5).map(|i|((seed>>(i*2))%4) as usize).collect::<Vec<_>>()})
        }
        "anomaly" => {
            let mut symbols = vec!["◇"; 12];
            symbols[n % 12] = "◈";
            json!({"symbols":symbols,"odd_index":n%12})
        }
        "circuits" | "route_power" => {
            // A snake traverses all nine cells, from the west of 0 to the east of 8.
            let path = [10u64, 10, 12, 6, 10, 9, 3, 10, 10];
            let solution: Vec<u64> = (0..9).map(|i| (seed >> (i * 2)) % 4).collect();
            let masks: Vec<u64> = path
                .iter()
                .zip(&solution)
                .map(|(&m, &r)| rotate(m, (4 - r) % 4))
                .collect();
            let rotations: Vec<u64> = solution
                .iter()
                .enumerate()
                .map(|(i, r)| (r + 1 + (i as u64 % 3)) % 4)
                .collect();
            json!({"rotations":rotations,"solution":solution,"masks":masks})
        }
        "password" => {
            let a = 1 + seed % 8;
            let b = 1 + (seed / 8) % 8;
            let c = 1 + (seed / 64) % 8;
            json!({"clues":[format!("The first digit is {} minus 1.",a+1),format!("The second digit is {} plus 1.",b-1),format!("The last digit is half of {}.",c*2)],"answer":format!("{a}{b}{c}")})
        }
        "mixing" => {
            let a = 25 + seed % 51;
            json!({"target_a":a,"target_b":100-a,"tolerance":3,"hold_ms":1000})
        }
        "transfer" => json!({"required_ms":5000+seed%3*1000,"tolerance":14}),
        "fault" => json!({"fault_index":seed%4,"lights":4}),
        "frequency" => json!({"target":25+seed%51,"tolerance":2,"hold_ms":1000}),
        "breaker" => json!({"code":format!("{:03}",100+seed%900)}),
        "timing" => {
            let start = 40 + seed % 11;
            json!({"zone_start":start,"zone_end":start+8,"required_hits":3})
        }
        "switches" => {
            let mut order = vec![2, 0, 3, 1];
            order.rotate_left(n % 4);
            json!({"order":order})
        }
        "cargo" => {
            let mut items = vec![
                json!({"label":"Wrench","category":0}),
                json!({"label":"Food","category":1}),
                json!({"label":"Hammer","category":0}),
                json!({"label":"Water","category":1}),
                json!({"label":"Pliers","category":0}),
                json!({"label":"Blanket","category":1}),
            ];
            let len = items.len();
            items.rotate_left(n % len);
            json!({"categories":["Tools","Supplies"],"items":items})
        }
        _ => unreachable!(),
    }
}
fn rotate(mask: u64, turns: u64) -> u64 {
    ((mask << turns) | (mask >> (4 - turns))) & 15
}
fn ints(value: &Value) -> Option<Vec<u64>> {
    value.as_array()?.iter().map(Value::as_u64).collect()
}
pub fn valid(kind: &str, c: &Value, r: &Value) -> bool {
    match kind {
        "wires" => {
            let Some(v) = ints(&r["connections"]) else {
                return false;
            };
            let Some(order) = ints(&c["right_order"]) else {
                return false;
            };
            v.len() == 4
                && v.iter()
                    .enumerate()
                    .all(|(i, &j)| order.get(j as usize) == Some(&(i as u64)))
        }
        "code" | "password" | "breaker" => r["answer"].as_str().is_some_and(|s| {
            Some(s) == c[if kind == "password" { "answer" } else { "code" }].as_str()
        }),
        "signal" | "frequency" => {
            r["value"].as_f64().is_some_and(|v| {
                (v - c["target"].as_f64().unwrap()).abs() <= c["tolerance"].as_f64().unwrap()
            }) && r["held_ms"]
                .as_u64()
                .is_some_and(|v| v >= c["hold_ms"].as_u64().unwrap())
        }
        "battery" | "timing" => r["hits"].as_array().is_some_and(|hits| {
            Some(hits.len() as u64) == c["required_hits"].as_u64()
                && hits.iter().all(|v| {
                    v.as_f64().is_some_and(|x| {
                        x >= c["zone_start"].as_f64().unwrap()
                            && x <= c["zone_end"].as_f64().unwrap()
                    })
                })
        }),
        "memory" => r["sequence"] == c["sequence"],
        "anomaly" => r["index"].as_u64() == c["odd_index"].as_u64(),
        "circuits" | "route_power" => circuit_valid(c, r),
        "mixing" => {
            [("a", "target_a"), ("b", "target_b")]
                .iter()
                .all(|&(value, target)| {
                    r[value].as_f64().is_some_and(|v| {
                        (v - c[target].as_f64().unwrap()).abs() <= c["tolerance"].as_f64().unwrap()
                    })
                })
                && r["held_ms"]
                    .as_u64()
                    .is_some_and(|v| v >= c["hold_ms"].as_u64().unwrap())
        }
        "transfer" => {
            r["progress"].as_f64() == Some(100.0)
                && r["held_ms"]
                    .as_u64()
                    .is_some_and(|v| v >= c["required_ms"].as_u64().unwrap())
        }
        "fault" => r["index"]
            .as_u64()
            .is_some_and(|v| Some(v) == c["fault_index"].as_u64()),
        "switches" => ints(&r["sequence"]).is_some_and(|v| Some(v) == ints(&c["order"])),
        "cargo" => ints(&r["choices"]).is_some_and(|v| {
            v == c["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| item["category"].as_u64().unwrap())
                .collect::<Vec<_>>()
        }),
        _ => false,
    }
}
fn circuit_valid(c: &Value, r: &Value) -> bool {
    let Some(turns) = ints(&r["rotations"]) else {
        return false;
    };
    if turns.len() != 9 || turns.iter().any(|r| *r > 3) {
        return false;
    }
    let masks = ints(&c["masks"]).unwrap();
    let board: Vec<u64> = masks
        .iter()
        .zip(turns)
        .map(|(&m, r)| rotate(m, r))
        .collect();
    if board[0] & 8 == 0 || board[8] & 2 == 0 {
        return false;
    }
    let mut seen = [false; 9];
    let mut queue = vec![0usize];
    seen[0] = true;
    while let Some(i) = queue.pop() {
        for (bit, opposite, j) in [
            (1, 4, i.checked_sub(3)),
            (2, 8, if i % 3 < 2 { Some(i + 1) } else { None }),
            (4, 1, if i < 6 { Some(i + 3) } else { None }),
            (8, 2, if i % 3 > 0 { Some(i - 1) } else { None }),
        ] {
            if let Some(j) = j {
                if !seen[j] && board[i] & bit != 0 && board[j] & opposite != 0 {
                    seen[j] = true;
                    queue.push(j)
                }
            }
        }
    }
    seen[8]
}
