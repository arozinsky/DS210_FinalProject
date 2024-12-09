use std::{collections::HashMap, fs::File, io::{self, BufReader, BufRead}};

#[derive(Debug)]
struct Player {
    name: String,
    positions: Vec<Position>,  
    metrics: Vec<f64>,
}

#[derive(Debug)]
enum Position {
    Centers,
    Wings,
    Defensive,
}

fn clean_fields(line: &str) -> Option<(String, Vec<Position>, Vec<f64>)> {
    let fields: Vec<&str> = line.split(',').map(|s| s.trim_matches('"')).collect();

    if fields.len() < 33 {
        eprintln!("Skipping row due to insufficient fields: {}", line);
        return None;
    }

    let player_name = fields[1].to_string();
    let position_str = fields[3].to_string();

    let mut positions = Vec::new();
    let metrics = match position_str.as_str() {
        "C" => {
            positions.push(Position::Centers);
            vec![
                fields.get(34).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0), // faceoff%
                fields.get(10).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0), // total points
                fields.get(28).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0), // takeaways
                fields.get(8).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0),  // 1st assist
                fields.get(11).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0), // ipp
            ]
        },
        "L" | "R" => {
            positions.push(Position::Wings);
            vec![
                fields.get(6).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0),  // goals
                fields.get(13).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0), // sh%
                fields.get(19).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0), // rush
                fields.get(10).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0), // total points
                fields.get(29).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0), // hits
            ]
        },
        "D" => {
            positions.push(Position::Defensive);
            vec![
                fields.get(29).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0), // Hits
                fields.get(31).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0), // Blocks
                fields.get(28).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0), // Takeaways
                fields.get(20).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0), // Rebounds
                fields.get(19).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0), // Rush
            ]
        },
        _ => {
            eprintln!("Skipping row with invalid position for player {}", player_name);
            return None; // Skip if the position is not valid
        }
    };

    Some((player_name, positions, metrics))
}

fn normalize_metrics(players: &mut HashMap<String, Player>) {
    let mut max_metrics = vec![0.0; 5];

    for player in players.values() {
        for (i, &metric) in player.metrics.iter().enumerate() {
            if metric > max_metrics[i] {
                max_metrics[i] = metric;
            }
        }
    }

    for player in players.values_mut() {
        for (i, metric) in player.metrics.iter_mut().enumerate() {
            if max_metrics[i] > 0.0 {
                *metric /= max_metrics[i];
            }
        }
    }
}

fn calculate_score(position: &Position, metrics: &[f64]) -> f64 {
    let (weights, scaling_factor) = match position {
        Position::Centers => (&[0.25, 0.3, 0.15, 0.2, 0.1], 5.0),
        Position::Wings => (&[0.35, 0.2, 0.2, 0.2, 0.05], 5.0),
        Position::Defensive => (&[0.15, 0.3, 0.15, 0.2, 0.2], 5.0),
    };

    if metrics.len() != weights.len() {
        println!("Mismatch between metrics length and weights length: {:?}", metrics);
        return 0.0;
    }

    let weighted_sum: f64 = metrics
        .iter()
        .zip(weights.iter())
        .map(|(metric, weight)| metric * weight)
        .sum();

    let scaled_score = scaling_factor * weighted_sum; // Scale down further
    println!("Scaled score (before sigmoid): {}", scaled_score);

    // Apply sigmoid and clamp the result to a valid range
    (100.0 / (1.0 + (-scaled_score).exp())).clamp(0.0, 100.0)
}

fn main() -> io::Result<()> {
    let file = File::open("Player Season Totals - Natural Stat Trick.csv")?;
    let reader = BufReader::new(file);

    let mut players: HashMap<String, Player> = HashMap::new();

    // Process each line and insert valid players
    for line in reader.lines() {
        let line = line?;
        
        if let Some((player_name, positions, metrics)) = clean_fields(&line) {
            players.insert(player_name.clone(), Player { name: player_name, positions, metrics });
        } else {
            eprintln!("Skipping invalid or malformed row.");
        }
    }    

    // Normalize metrics for each player
    normalize_metrics(&mut players);

    // Calculate scores for each player and store them
    let mut scores: HashMap<String, f64> = HashMap::new();
    for (name, player) in &players {
        let total_score: f64 = player
            .positions
            .iter()
            .map(|position| calculate_score(position, &player.metrics))
            .sum();
        
        let average_score = total_score / player.positions.len() as f64;
        scores.insert(name.clone(), average_score);
    }

    // Sort players by score and display the top 10
    let mut sorted_players: Vec<_> = scores.iter().collect();
    sorted_players.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    println!("Top 10 Players:");
    for (name, score) in sorted_players.iter().take(10) {
        println!("{}: {:.2}/100", name, score);
    }

    // Allow user to input a player's name to get their score
    let mut input = String::new();
    println!("\nEnter a player name to get their score:");
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    match scores.iter().find(|(name, _)| name.to_lowercase() == input) {
        Some((name, score)) => println!("{}: {:.2}/100", name, score),
        None => println!("Player not found."),
    }

    Ok(())
}