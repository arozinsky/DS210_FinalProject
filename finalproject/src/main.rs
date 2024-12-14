mod cleaning;
use std::{collections::HashMap, fs::File, io::{self, BufReader, BufRead, stdin}};
use cleaning::{clean_fields, normalize_metrics, Player, Position};
use crate::cleaning::process_file;

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

                println!("\nCurrent Rating: {:.2}%", total_score);
            }
            None => println!("Player '{}' not found. Please try again.", player_name),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, io::Write, fs::File};

    #[test]
    fn test_clean_fields_center() {
    let input = r#"1,"Player One","C",,,,33.0,7.0,10.0,,,,,,,,,,,,,,,27.0,,,,30.0,18.0,,,,,,,,,,,,,,,,,"#;
    let result = clean_fields(input);
    assert!(result.is_some(), "Failed to clean fields for valid input");
    
    let (name, positions, metrics) = result.unwrap();
    assert_eq!(name, "Player One");
    assert_eq!(positions, vec![Position::Center]);
    assert!(metrics.contains_key(&Position::Center));
    }


    #[test]
    fn test_normalize_metrics() {
        let mut players: HashMap<String, Player> = HashMap::new();
        players.insert(
            "Player A".to_string(),
            Player {
                name: "Player A".to_string(),
                positions: vec![Position::Wing],
                metrics: HashMap::from([(
                    Position::Wing,
                    vec![10.0, 20.0, 30.0],
                )]),
            },
        );
        players.insert(
            "Player B".to_string(),
            Player {
                name: "Player B".to_string(),
                positions: vec![Position::Wing],
                metrics: HashMap::from([(
                    Position::Wing,
                    vec![20.0, 10.0, 40.0],
                )]),
            },
        );

        normalize_metrics(&mut players);

        let wing_metrics_a = &players["Player A"].metrics[&Position::Wing];
        let wing_metrics_b = &players["Player B"].metrics[&Position::Wing];

        assert_eq!(wing_metrics_a, &[0.5, 1.0, 0.75]);
        assert_eq!(wing_metrics_b, &[1.0, 0.5, 1.0]);
    }

    #[test]
    fn test_calculate_score() {
        let metrics = vec![0.5, 1.0, 0.75, 0.8, 0.9];
        let score = calculate_score(&Position::Wing, &metrics);
        assert!(score > 0.0 && score <= 100.0);
    }

    #[test]
    fn test_process_file_skip_header() {
    let input = r#"1,"Player One","C",,,,33.0,7.0,10.0,,,,,,,,,,,,,,,27.0,,,,30.0,18.0,
                   2,"Player Two","L",,,,22.0,5.0,12.0,,,,,,,,,,,,,,,21.0,,,,25.0,15.0,
                   3,"Player Three","D",,,,20.0,10.0,5.0,,,,,,,,,,,,,,,18.0,,,,28.0,12.0,"#;
    let file_path = "test.csv";
    
    let mut file = std::fs::File::create(file_path).unwrap();
    file.write_all(input.as_bytes()).unwrap();
    
    let players = process_file(file_path).unwrap();
    
    assert_eq!(players.len(), 0);
    }
}