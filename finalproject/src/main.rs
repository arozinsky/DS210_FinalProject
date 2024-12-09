use std::{collections::HashMap, fs::File, io::{self, BufReader, BufRead}};

#[derive(Debug)]
struct Player {
    name: String,
    position: Position,
    metrics: Vec<f64>,
}

#[derive(Debug)]
enum Position {
    Offensive,  
    Defensive,  
}

fn clean_fields(line: &str) -> Vec<String> {
    line.split(',')
        .map(|s| s.trim_matches('"').to_string())
        .collect()
}

fn calculate_score(position: &Position, metrics: &[f64]) -> f64 {
    let (weights, _) = match position {
        Position::Offensive => (&[0.4, 0.3, 0.2, 0.1], 1.0),
        Position::Defensive => (&[0.2, 0.3, 0.2, 0.3], 1.0),
    };

    if metrics.len() != weights.len() {
        println!("Mismatch between metrics length and weights length: {:?}", metrics);
        return 0.0;
    }

    let weighted_score: f64 = metrics.iter().zip(weights.iter())
        .map(|(metric, weight)| metric * weight)
        .sum();


    println!("Weighted score (before sigmoid): {}", weighted_score);

    100.0 / (1.0 + (-weighted_score).exp())
}


fn main() -> io::Result<()> {
    let file = File::open("Player Season Totals - Natural Stat Trick.csv")?;
    let reader = BufReader::new(file);

    let mut players: HashMap<String, Player> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let fields = clean_fields(&line);

        if fields.len() < 33 { continue; }

        let player_name = fields[1].to_string();
        let position_str = fields[2].to_string();

        let position = match position_str.as_str() {
            "C" | "L" | "R" => Position::Offensive,
            "D" => Position::Defensive,            
            _ => continue,
        };

        let metrics = match position {
            Position::Offensive => {
                vec![
                    fields[6].parse::<f64>().unwrap_or(0.0),
                    fields[7].parse::<f64>().unwrap_or(0.0),
                    fields[12].parse::<f64>().unwrap_or(0.0),
                    fields[5].parse::<f64>().unwrap_or(0.0),
                ]
            },
            Position::Defensive => {
                vec![
                    fields[29].parse::<f64>().unwrap_or(0.0),
                    fields[31].parse::<f64>().unwrap_or(0.0),
                    fields[5].parse::<f64>().unwrap_or(0.0),  
                    fields[28].parse::<f64>().unwrap_or(0.0),
                ]
            },
        };

        players.insert(player_name.clone(), Player { name: player_name, position, metrics });
    }

    let mut scores: HashMap<String, f64> = HashMap::new();
    for (name, player) in &players {
        let score = calculate_score(&player.position, &player.metrics);
        scores.insert(name.clone(), score);
    }

    let mut sorted_players: Vec<_> = scores.iter().collect();
    sorted_players.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("Top 10 Players:");
    for (name, score) in sorted_players.iter().take(10) {
        println!("{}: {:.2}/100", name, score);
    }

    let mut input = String::new();
    println!("\nEnter a player name to get their score:");
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    match scores.get(input) {
        Some(score) => println!("{}: {:.2}/100", input, score),
        None => println!("Player not found."),
    }

    Ok(())
}