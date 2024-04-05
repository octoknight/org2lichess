use crate::db::{Membership, OrgDbClient};
use crate::lichess;
use crate::org;
use crate::textlog;
use crate::types::*;
use chrono_tz::Tz;
use std::thread;

fn find_expired_members(
    db: &OrgDbClient,
    timezone: Tz,
    month: u32,
    day: u32,
) -> Result<Vec<Membership>, ErrorBox> {
    let current_year = org::current_year(timezone);

    let year = if org::is_past_renewal(current_year, timezone, month, day) {
        current_year
    } else {
        current_year - 1
    };

    db.get_members_with_at_most_expiry_year(year)
}

fn clean_expired_members(
    expired_members: Vec<Membership>,
    delay_ms: u64,
    db: &OrgDbClient,
    http_client: &reqwest::blocking::Client,
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
            match db.remove_membership(&member.org_id) {
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

#[allow(clippy::too_many_arguments)]
fn find_and_clean_expired(
    delay_ms: u64,
    db: &OrgDbClient,
    http_client: &reqwest::blocking::Client,
    lichess_domain: &str,
    team_id: &str,
    api_token: &str,
    timezone: Tz,
    renewal_month: u32,
    renewal_day: u32,
) {
    match find_expired_members(&db, timezone, renewal_month, renewal_day) {
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

#[allow(clippy::too_many_arguments)]
pub fn launch(
    db_client: OrgDbClient,
    lichess_domain: String,
    team_id: String,
    api_token: String,
    interval_seconds: u64,
    timezone: Tz,
    renewal_month: u32,
    renewal_day: u32,
) {
    thread::spawn(move || {
        let http_client = reqwest::blocking::Client::new();

        loop {
            println!("Finding and cleaning expired members...");

            find_and_clean_expired(
                1000,
                &db_client,
                &http_client,
                &lichess_domain,
                &team_id,
                &api_token,
                timezone,
                renewal_month,
                renewal_day,
            );

            thread::sleep(std::time::Duration::from_secs(interval_seconds));
        }
    });
}
