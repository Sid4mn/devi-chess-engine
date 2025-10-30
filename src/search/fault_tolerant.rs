use crate::board::{Board, BoardRepresentation};
use crate::moves::generate_legal_moves;
use crate::search::minimax::alphabeta;
use crate::types::*;
use rayon::prelude::*;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::search::minimax::MATE_SCORE;

#[derive(Debug)]
struct CrashInfo {
    thread_id: Option<usize>,
    root_index: usize,
    move_attempted: String,
    panic_message: String,
    timestamp: SystemTime,
}

pub fn search_root_fault(board: &mut Board, depth: u32, inject_panic_at: Option<usize>) -> (Move, i32) {
    let current_color = board.to_move();
    let moves = generate_legal_moves(board, current_color);

    if moves.is_empty() {
        let dummy_move = Move::new(Square(0), Square(0), None, None);
        let score = if board.is_in_check(current_color) {
            -MATE_SCORE // We're checkmated
        } else {
            0 // Stalemate
        };
        return (dummy_move, score);
    }

    let crash_log: Arc<Mutex<Vec<CrashInfo>>> = Arc::new(Mutex::new(Vec::new()));

    let successful_results: Vec<(Move, i32)> = moves
        .par_iter()
        .enumerate()
        .filter_map(|(index, mv)| {
            let crash_log_clone = Arc::clone(&crash_log);

            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                let mut local_board = board.clone();

                if inject_panic_at == Some(index) {
                    panic!("Injected fault at root move {}", index);
                }

                let undo = local_board.make_move(mv);
                let score = -alphabeta(
                    &mut local_board,
                    depth.saturating_sub(1),
                    i32::MIN + 1,
                    i32::MAX - 1,
                    false,
                );
                local_board.unmake_move(mv, undo);

                (*mv, score)
            }));

            match result {
                Ok(move_score) => Some(move_score),
                Err(panic_payload) => {
                    let panic_message = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic".to_string()
                    };

                    let crash_info = CrashInfo {
                        thread_id: Some(index),
                        root_index: index,
                        move_attempted: format!("{:?}", mv),
                        panic_message: panic_message.clone(),
                        timestamp: SystemTime::now(),
                    };

                    if let Ok(mut log) = crash_log_clone.lock() {
                        log.push(crash_info);
                    }

                    eprintln!("Worker {} panicked: {}", index, panic_message);

                    None
                }
            }
        })
        .collect();

    if let Ok(log) = crash_log.lock() {
        if !log.is_empty() {
            export_crash_logs(&log);
        }
    }

    if successful_results.is_empty() {
        eprintln!("All workers failed! Returning dummy move.");
        let dummy_move = Move::new(Square(0), Square(0), None, None);
        return (dummy_move, -MATE_SCORE);
    }

    successful_results
        .into_iter()
        .max_by_key(|&(_, score)| score)
        .expect("We already checked that results is non-empty")
}

fn export_crash_logs(crashes: &[CrashInfo]) {
    use std::fs::{create_dir_all, File};
    use std::io::Write;

    // Create crashes directory
    if let Err(e) = create_dir_all("crashes") {
        eprintln!("Failed to create crashes directory: {}", e);
        return;
    }

    // Generate timestamp for filename
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let filename = format!("crashes/crash_{}.json", timestamp);

    if let Ok(mut file) = File::create(&filename) {
        writeln!(file, "[").unwrap();
        for (i, crash) in crashes.iter().enumerate() {
            writeln!(file, "  {{").unwrap();
            writeln!(file, "    \"thread_id\": {:?},", crash.thread_id).unwrap();
            writeln!(file, "    \"root_index\": {},", crash.root_index).unwrap();
            writeln!(
                file,
                "    \"move_attempted\": \"{}\",",
                crash.move_attempted
            )
            .unwrap();
            writeln!(file, "    \"panic_message\": \"{}\",", crash.panic_message).unwrap();
            writeln!(
                file,
                "    \"timestamp\": {:?}",
                crash
                    .timestamp
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            )
            .unwrap();
            write!(file, "  }}").unwrap();
            if i < crashes.len() - 1 {
                writeln!(file, ",").unwrap();
            } else {
                writeln!(file).unwrap();
            }
        }
        writeln!(file, "]").unwrap();

        eprintln!("Crash logs exported to {}", filename);
    }
}
