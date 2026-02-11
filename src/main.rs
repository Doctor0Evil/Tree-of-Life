mod simulation;
mod ledger;
mod utils;

use clap::{Parser, Subcommand};
use crate::simulation::MicroSociety;
use crate::ledger::ChurchAccountState;
use std::error::Error;

#[derive(Parser)]
#[command(name = "microsociety_sim")]
#[command(about = "Non-actuating MicroSociety Tree-of-Life sandbox")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a bounded number of simulation cycles and print a summary.
    SimulateCycles {
        #[arg(long)]
        cycles: usize,
        #[arg(long)]
        agent_count: usize,
    },
    /// Compute advisory rights metrics for a given agent ID from a fresh in-memory ledger.
    ComputeRights {
        #[arg(long)]
        agent_id: String,
        #[arg(long, default_value_t = 20)]
        agent_count: usize,
        #[arg(long, default_value_t = 10)]
        warmup_cycles: usize,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::SimulateCycles { cycles, agent_count } => {
            let mut society = MicroSociety::new(agent_count);
            for _ in 0..cycles {
                society.simulate_cycle();
            }
            println!(
                "Simulation completed for {} agents over {} cycles (sandbox-only).",
                agent_count, cycles
            );
        }
        Commands::ComputeRights {
            agent_id,
            agent_count,
            warmup_cycles,
        } => {
            let mut society = MicroSociety::new(agent_count);
            for _ in 0..warmup_cycles {
                society.simulate_cycle();
            }

            if let Some(state) = ChurchAccountState::compute_from_ledger(&society.ledger, &agent_id)
            {
                println!(
                    "Advisory Rights-to-Exist score for {}: {:.3}",
                    agent_id, state.existence_rights_score
                );
                let grant = state.compute_grant_amount();
                if grant > 0.0 {
                    println!(
                        "Advisory eco_grant suggestion (symbolic units): {:.3}",
                        grant
                    );
                }
            } else {
                println!("No ledger data for agent {}", agent_id);
            }
        }
    }

    Ok(())
}
