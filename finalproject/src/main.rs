mod cleaning;
use std::{collections::HashMap, fs::File, io::{self, BufReader, BufRead, stdin}};
use cleaning::{clean_fields, normalize_metrics, Player, Position};

fn calculate_score(position: &Position, metrics: &[f64]) -> f64 {
    let (weights, scaling_factor) = match position {
        Position::Center => (&[0.25, 0.3, 0.15, 0.2, 0.1], 5.0),
        Position::Wing => (&[0.35, 0.25, 0.15, 0.2, 0.05], 5.0),
        Position::Defense => (&[0.15, 0.3, 0.2, 0.2, 0.15], 5.0),
    };

    if metrics.len() != weights.len() || metrics.iter().any(|m| !m.is_finite()) {
        eprintln!("Invalid metrics for scoring: {:?}", metrics);
        return 0.0;
    }

    let weighted_sum: f64 = metrics
        .iter()
        .zip(weights.iter())
        .map(|(metric, weight)| metric * weight)
        .sum();

    let scaled_score = scaling_factor * weighted_sum; 

    (100.0 / (1.0 + (-scaled_score).exp())).clamp(0.0, 100.0)
}

fn main() -> io::Result<()> {
    let file = File::open("NHL.csv")?; 
    let reader = BufReader::new(file);

    let mut players: HashMap<String, Player> = HashMap::new();
    let mut skipped_rows = 0;
    let mut processed_rows = 0;

    for line in reader.lines() {
        let line = line?; 

        if let Some((player_name, positions, metrics)) = clean_fields(&line) {
            players.insert(player_name.clone(), Player { name: player_name, positions, metrics });
            processed_rows += 1;
        } else {
            skipped_rows += 1;
        }
    }

    println!("Processed rows: {}", processed_rows);
    println!("Skipped rows: {}", skipped_rows);

    normalize_metrics(&mut players);

    let mut position_groups: HashMap<Position, Vec<(String, f64)>> = HashMap::new();

    for (name, player) in &players {
        for position in &player.positions {
            if let Some(metrics_for_position) = player.metrics.get(position) {
                let score = calculate_score(position, metrics_for_position);
                position_groups
                    .entry(position.clone())
                    .or_insert_with(Vec::new)
                    .push((name.clone(), score));
            }
        }
    }

    for position in &[Position::Center, Position::Wing, Position::Defense] {
        if let Some(players_in_position) = position_groups.get_mut(position) {
            players_in_position.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            println!("\nTop Players in {:?} Position:", position);
            for (name, score) in players_in_position.iter().take(10) {
                println!("{}: {:.2}%", name, score);
            }
        }
    }

    let mut input = String::new(); 
    loop {
        println!("\nEnter a player name to get their score (or press Enter to exit):");

        input.clear();
        stdin().read_line(&mut input)?; 
        let player_name = input.trim().to_lowercase();

        if player_name.is_empty() {
            println!("Exiting...");
            break;
        }

        match players.iter().find(|(name, _)| name.to_lowercase() == player_name) {
            Some((player_name, player)) => {
                let mut total_score = 0.0;
                println!("Player: {}", player_name);

                for position in &player.positions {
                    if let Some(metrics_for_position) = player.metrics.get(position) {
                        let score = calculate_score(position, metrics_for_position);
                        total_score += score; 

                        println!("\nStats for {} at {:?}:", player_name, position);
                        let metric_names = match position {
                            Position::Center => vec!["Faceoffs %", "Total Points", "Takeaways", "First Assists", "IPP"],
                            Position::Wing => vec!["Goals", "SH%", "Rush Attempts", "Total Points", "Hits"],
                            Position::Defense => vec!["Hits", "Shots Blocked", "Takeaways", "Rebounds Created", "Rush Attempts"],
                        };

                        for (i, &metric) in metrics_for_position.iter().enumerate() {
                            println!("{}: {:.2}", metric_names[i], metric);
                        }
                    }
                }

                println!("\nFinal Score: {:.2}%", total_score);
            }
            None => println!("Player '{}' not found. Please try again.", player_name),
        }
    }

    Ok(())
}