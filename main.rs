use std::{collections::HashMap, fs::File, io::{self, BufReader, BufRead}};

// Function to clean each CSV line (remove quotes)
fn clean_fields(line: &str) -> Vec<String> {
    line.split(',')
        .map(|s| s.trim_matches('"').to_string()) // Remove quotation marks
        .collect()
}

// Function to calculate a player's score based on metrics (simplified)
fn calculate_score(position: &str, metrics: &[f64]) -> f64 {
    let (weights, _, _) = match position {
        "C" | "L" | "R" => (&[0.4, 0.3, 0.2, 0.1], 1.0, 0.5), // Offensive
        "D" => (&[0.1, 0.3, 0.3, 0.3], 1.0, 0.5),             // Defensive
        _ => return 0.0, // Default score for unknown positions
    };

    if metrics.len() != weights.len() {
        println!("Mismatch between metrics length and weights length: {:?}", metrics);
        return 0.0;
    }

    // Print metrics and weights for debugging
    println!("Metrics: {:?}", metrics);
    println!("Weights: {:?}", weights);

    // Use the sigmoid function to calculate a weighted score (simplified example)
    let weighted_score: f64 = metrics.iter().zip(weights.iter())
        .map(|(metric, weight)| metric * weight)
        .sum();

    println!("Weighted Score: {}", weighted_score);

    100.0 / (1.0 + (-weighted_score).exp()) // Apply sigmoid function
}

fn main() -> io::Result<()> {
    let file = File::open("Player Season Totals - Natural Stat Trick.csv")?; // Update path if necessary
    let reader = BufReader::new(file);

    let mut players: HashMap<String, f64> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let fields = clean_fields(&line); // Clean the line using the clean_fields function

        // Print fields for debugging
        println!("Fields: {:?}", fields);

        if fields.len() < 5 { continue; } // Skip rows with insufficient columns

        let player_name = fields[1].to_string(); // Player's name
        let position = fields[2].to_string(); // Player's position
        let metrics: Vec<f64> = fields[3..]
            .iter()
            .filter_map(|x| x.parse::<f64>().ok()) // Parse the metrics to f64
            .collect();

        // Print parsed metrics for debugging
        println!("Parsed metrics for {}: {:?}", player_name, metrics);

        if metrics.is_empty() {
            println!("No valid metrics found for player: {}", player_name);
        }

        let score = calculate_score(&position, &metrics);
        players.insert(player_name, score);
    }

    let mut sorted_players: Vec<_> = players.iter().collect();
    sorted_players.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("Top 10 Players:");
    for (name, score) in sorted_players.iter().take(10) {
        println!("{}: {:.2}/100", name, score);
    }

    // User input for player score
    let mut input = String::new();
    println!("\nEnter a player name to get their score:");
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    match players.get(input) {
        Some(score) => println!("{}: {:.2}/100", input, score),
        None => println!("Player not found."),
    }

    Ok(())
}
