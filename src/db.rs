use crate::types::*;
use postgres::{Client, NoTls};
use serde::Serialize;
use std::sync::{RwLock, RwLockWriteGuard};

#[derive(Serialize)]
pub struct Membership {
    pub ecf_id: i32,
    pub lichess_id: String,
    pub exp_year: i32,
}

pub fn connect(connection_string: &str) -> Result<Client, postgres::Error> {
    Client::connect(connection_string, NoTls)
}

pub trait EcfDbClient {
    fn w(&self) -> Result<RwLockWriteGuard<'_, Client>, ErrorBox>;
    fn register_member(
        &self,
        ecf_id: i32,
        lichess_id: &str,
        exp_year: i32,
    ) -> Result<u64, ErrorBox>;
    fn get_member_for_ecf_id(&self, ecf_id: i32) -> Result<Option<Membership>, ErrorBox>;
    fn get_member_for_lichess_id(&self, lichess_id: &str) -> Result<Option<Membership>, ErrorBox>;
    fn lichess_member_has_ecf(&self, lichess_id: &str) -> Result<bool, ErrorBox>;
    fn remove_membership(&self, ecf_id: i32) -> Result<u64, ErrorBox>;
    fn get_members(&self) -> Result<Vec<Membership>, ErrorBox>;
    fn get_members_with_at_most_expiry_year(&self, year: i32) -> Result<Vec<Membership>, ErrorBox>;
}

fn extract_one_membership(rows: &Vec<postgres::row::Row>) -> Option<Membership> {
    rows.get(0).map(|row| {
        let member = Membership {
            ecf_id: row.get(0),
            lichess_id: row.get(1),
            exp_year: row.get(2),
        };
        member
    })
}

impl EcfDbClient for RwLock<Client> {
    fn w(&self) -> Result<RwLockWriteGuard<'_, Client>, ErrorBox> {
        match self.write() {
            Ok(client) => Ok(client),
            _ => Err(".write() failed".into()),
        }
    }

    fn register_member(
        &self,
        ecf_id: i32,
        lichess_id: &str,
        exp_year: i32,
    ) -> Result<u64, ErrorBox> {
        let mut client = self.w()?;
        client.execute(
            "DELETE FROM memberships WHERE ecfid = $1 OR lichessid = $2",
            &[&ecf_id, &lichess_id],
        )?;
        let result = client.execute(
            "INSERT INTO memberships (ecfid, lichessid, exp) VALUES ($1, $2, $3);",
            &[&ecf_id, &lichess_id, &exp_year],
        )?;
        Ok(result)
    }

    fn get_member_for_ecf_id(&self, ecf_id: i32) -> Result<Option<Membership>, ErrorBox> {
        let rows = self.w()?.query(
            "SELECT ecfid, lichessid, exp FROM memberships WHERE ecfid = $1",
            &[&ecf_id],
        )?;
        Ok(extract_one_membership(&rows))
    }

    fn get_member_for_lichess_id(&self, lichess_id: &str) -> Result<Option<Membership>, ErrorBox> {
        let rows = self.w()?.query(
            "SELECT ecfid, lichessid, exp FROM memberships WHERE lichessid = $1",
            &[&lichess_id],
        )?;
        Ok(extract_one_membership(&rows))
    }

    fn lichess_member_has_ecf(&self, lichess_id: &str) -> Result<bool, ErrorBox> {
        self.get_member_for_lichess_id(lichess_id)
            .map(|member| member.is_some())
    }

    fn remove_membership(&self, ecf_id: i32) -> Result<u64, ErrorBox> {
        let result = self
            .w()?
            .execute("DELETE FROM memberships WHERE ecfid = $1", &[&ecf_id])?;
        Ok(result)
    }

    fn get_members(&self) -> Result<Vec<Membership>, ErrorBox> {
        let mut members: Vec<Membership> = vec![];
        for row in self
            .w()?
            .query("SELECT ecfid, lichessid, exp FROM memberships", &[])?
        {
            members.push(Membership {
                ecf_id: row.get(0),
                lichess_id: row.get(1),
                exp_year: row.get(2),
            });
        }
        Ok(members)
    }

    fn get_members_with_at_most_expiry_year(&self, year: i32) -> Result<Vec<Membership>, ErrorBox> {
        let mut members: Vec<Membership> = vec![];
        for row in self.w()?.query(
            "SELECT ecfid, lichessid, exp FROM memberships WHERE exp <= $1",
            &[&year],
        )? {
            members.push(Membership {
                ecf_id: row.get(0),
                lichess_id: row.get(1),
                exp_year: row.get(2),
            });
        }
        Ok(members)
    }
}
