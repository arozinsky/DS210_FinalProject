use std::{collections::HashMap, fs::File, io::{self, BufReader, BufRead}};

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub positions: Vec<Position>,
    pub metrics: HashMap<Position, Vec<f64>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Position {
    Center,
    Wing,
    Defense,
}

pub fn process_file(file_path: &str) -> io::Result<HashMap<String, Player>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut players: HashMap<String, Player> = HashMap::new();

    let mut lines = reader.lines();
    
    lines.next();

    for line in lines {
        let line = line?;

        if let Some((player_name, positions, metrics)) = clean_fields(&line) {
            players.insert(player_name.clone(), Player { name: player_name, positions, metrics });
        }
    }

    Ok(players)
}

pub fn clean_fields(line: &str) -> Option<(String, Vec<Position>, HashMap<Position, Vec<f64>>)> {
    let fields: Vec<&str> = line.split(',').map(|f| f.trim().trim_matches('"')).collect();

    if fields.len() < 34 {
        eprintln!("Row skipped: Insufficient fields ({}/{}) - {}", fields.len(), 34, line);
        return None;
    }

    let player_name = fields[1].to_string(); 
    let position_str = fields[2]; 

    if player_name.is_empty() || position_str.is_empty() {
        eprintln!("Row skipped: Missing player name or position - {}", line);
        return None;
    }

    let mut positions = Vec::new();
    let mut metrics = HashMap::new();

    let mut has_left_wing = false;
    let mut has_right_wing = false;

    for pos in position_str.split('/') {
        match pos {
            "C" => {
                positions.push(Position::Center);
                metrics.insert(
                    Position::Center,
                    vec![ 
                        fields[33].parse::<f64>().unwrap_or_else(|_| default_metric("Faceoffs %", &player_name)),
                        fields[9].parse::<f64>().unwrap_or_else(|_| default_metric("Total Points", &player_name)),
                        fields[27].parse::<f64>().unwrap_or_else(|_| default_metric("Takeaways", &player_name)),
                        fields[7].parse::<f64>().unwrap_or_else(|_| default_metric("First Assists", &player_name)),
                        fields[10].parse::<f64>().unwrap_or_else(|_| default_metric("IPP", &player_name))
                    ]
                );
            },
            "L" | "R" => {
                if pos == "L" {
                    has_left_wing = true;
                }
                if pos == "R" {
                    has_right_wing = true;
                }

                positions.push(Position::Wing);
                metrics.entry(Position::Wing).or_insert_with(Vec::new).extend(
                    vec![ 
                        fields[5].parse::<f64>().unwrap_or_else(|_| default_metric("Goals", &player_name)),
                        fields[12].parse::<f64>().unwrap_or_else(|_| default_metric("SH%", &player_name)),
                        fields[18].parse::<f64>().unwrap_or_else(|_| default_metric("Rush Attempts", &player_name)),
                        fields[9].parse::<f64>().unwrap_or_else(|_| default_metric("Total Points", &player_name)),
                        fields[28].parse::<f64>().unwrap_or_else(|_| default_metric("Hits", &player_name))
                    ]
                );
            },
            "D" => {
                positions.push(Position::Defense);
                metrics.insert(
                    Position::Defense,
                    vec![ 
                        fields[28].parse::<f64>().unwrap_or_else(|_| default_metric("Hits", &player_name)),
                        fields[30].parse::<f64>().unwrap_or_else(|_| default_metric("Shots Blocked", &player_name)),
                        fields[27].parse::<f64>().unwrap_or_else(|_| default_metric("Takeaways", &player_name)),
                        fields[9].parse::<f64>().unwrap_or_else(|_| default_metric("Total Points", &player_name)),
                        fields[18].parse::<f64>().unwrap_or_else(|_| default_metric("Rush Attempts", &player_name))
                    ]
                );
            },
            _ => {
                eprintln!("Row skipped: Invalid position '{}' for player '{}'", pos, player_name);
                return None;
            }
        }
    }

    Some((player_name, positions, metrics))
}

pub fn default_metric(metric_name: &str, player_name: &str) -> f64 {
    0.0 
}

pub fn normalize_metrics(players: &mut HashMap<String, Player>) {
    let mut max_metrics: HashMap<Position, Vec<f64>> = HashMap::new();

    for player in players.values() {
        for (position, metrics) in &player.metrics {
            max_metrics.entry(position.clone()).or_insert_with(|| vec![0.0; metrics.len()]);
            for (i, &metric) in metrics.iter().enumerate() {
                if metric.is_finite() && metric > max_metrics[&position][i] {
                    max_metrics.get_mut(&position).unwrap()[i] = metric;
                }
            }
        }
    }

    for player in players.values_mut() {
        for (position, metrics) in &mut player.metrics {
            if let Some(max_vals) = max_metrics.get(position) {
                for (i, metric) in metrics.iter_mut().enumerate() {
                    if max_vals[i] > 0.0 && metric.is_finite() {
                        *metric /= max_vals[i];
                    } else {
                        *metric = 0.0; 
                    }
                }
            }
        }
    }
}
