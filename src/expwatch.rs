use crate::db::{EcfDbClient, Membership};
use crate::ecf;
use crate::lichess;
use crate::textlog;
use crate::types::*;
use postgres;
use reqwest;
use std::sync::RwLock;
use std::thread;

fn find_expired_members(db: &RwLock<postgres::Client>) -> Result<Vec<Membership>, ErrorBox> {
    let current_year = ecf::current_london_year();

    let year = if ecf::is_past_renewal(current_year) {
        current_year
    } else {
        current_year - 1
    };

    db.get_members_with_at_most_expiry_year(year)
}

fn clean_expired_members(
    expired_members: Vec<Membership>,
    delay_ms: u64,
    db: &RwLock<postgres::Client>,
    http_client: &reqwest::Client,
    lichess_domain: &str,
    team_id: &str,
    api_token: &str,
) {
    for member in expired_members {
        if lichess::kick_from_team(
            &http_client,
            &api_token,
            &lichess_domain,
            &team_id,
            &member.lichess_id,
        ) {
            match db.remove_membership(member.ecf_id) {
                Ok(_) => {
                    textlog::append_line_to(
                        "kick.log",
                        &format!("Successfully kicked {}", &member.lichess_id),
                    )
                    .unwrap_or(());
                    println!("Successfully kicked {}", &member.lichess_id);
                }
                _ => {
                    textlog::append_line_to(
                        "kick.error.log",
                        &format!("Could not Remove {} from db", &member.lichess_id),
                    )
                    .unwrap_or(());
                    println!("Could not remove {} from db", &member.lichess_id);
                }
            }
        } else {
            textlog::append_line_to(
                "kick.error.log",
                &format!("Could not kick {}", &member.lichess_id),
            )
            .unwrap_or(());
            println!("Could not kick {}", &member.lichess_id);
        }

        thread::sleep(std::time::Duration::from_millis(delay_ms));
    }
}

fn find_and_clean_expired(
    delay_ms: u64,
    db: &RwLock<postgres::Client>,
    http_client: &reqwest::Client,
    lichess_domain: &str,
    team_id: &str,
    api_token: &str,
) {
    match find_expired_members(&db) {
        Ok(expired) => clean_expired_members(
            expired,
            delay_ms,
            &db,
            &http_client,
            &lichess_domain,
            &team_id,
            &api_token,
        ),
        _ => {
            textlog::append_line_to("expiry.error.log", "Could not fetch expired users from db")
                .unwrap_or(());
            println!("Could not fetch expired users from db");
        }
    }
}

pub fn launch(
    db_client: RwLock<postgres::Client>,
    lichess_domain: String,
    team_id: String,
    api_token: String,
    interval_seconds: u64,
) {
    thread::spawn(move || {
        let http_client = reqwest::Client::new();

        loop {
            println!("Finding and cleaning expired members...");

            find_and_clean_expired(
                1000,
                &db_client,
                &http_client,
                &lichess_domain,
                &team_id,
                &api_token,
            );

            thread::sleep(std::time::Duration::from_secs(interval_seconds));
        }
    });
}
