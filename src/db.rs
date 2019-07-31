use crate::types::*;
use postgres::{Client, NoTls};
use serde::Serialize;
use std::sync::{RwLock, RwLockWriteGuard};

#[derive(Serialize)]
pub struct Membership {
    pub org_id: String,
    pub lichess_id: String,
    pub exp_year: i32,
}

pub fn connect(connection_string: &str) -> Result<Client, postgres::Error> {
    Client::connect(connection_string, NoTls)
}

pub trait OrgDbClient {
    fn w(&self) -> Result<RwLockWriteGuard<'_, Client>, ErrorBox>;
    fn register_member(
        &self,
        org_id: &str,
        lichess_id: &str,
        exp_year: i32,
    ) -> Result<u64, ErrorBox>;
    fn get_member_for_org_id(&self, org_id: &str) -> Result<Option<Membership>, ErrorBox>;
    fn get_member_for_lichess_id(&self, lichess_id: &str) -> Result<Option<Membership>, ErrorBox>;
    fn lichess_member_has_org(&self, lichess_id: &str) -> Result<bool, ErrorBox>;
    fn remove_membership(&self, org_id: &str) -> Result<u64, ErrorBox>;
    fn get_members(&self) -> Result<Vec<Membership>, ErrorBox>;
    fn get_members_with_at_most_expiry_year(&self, year: i32) -> Result<Vec<Membership>, ErrorBox>;
    fn referral_click(&self, lichess_id: &str) -> Result<u64, ErrorBox>;
    fn referral_count(&self) -> Result<i64, ErrorBox>;
}

fn extract_one_membership(rows: &Vec<postgres::row::Row>) -> Option<Membership> {
    rows.get(0).map(|row| {
        let member = Membership {
            org_id: row.get(0),
            lichess_id: row.get(1),
            exp_year: row.get(2),
        };
        member
    })
}

impl OrgDbClient for RwLock<Client> {
    fn w(&self) -> Result<RwLockWriteGuard<'_, Client>, ErrorBox> {
        match self.write() {
            Ok(client) => Ok(client),
            _ => Err(".write() failed".into()),
        }
    }

    fn register_member(
        &self,
        org_id: &str,
        lichess_id: &str,
        exp_year: i32,
    ) -> Result<u64, ErrorBox> {
        let mut client = self.w()?;
        client.execute(
            "DELETE FROM memberships WHERE orgid = $1 OR lichessid = $2",
            &[&org_id, &lichess_id],
        )?;
        let result = client.execute(
            "INSERT INTO memberships (orgid, lichessid, exp) VALUES ($1, $2, $3);",
            &[&org_id, &lichess_id, &exp_year],
        )?;
        Ok(result)
    }

    fn get_member_for_org_id(&self, org_id: &str) -> Result<Option<Membership>, ErrorBox> {
        let rows = self.w()?.query(
            "SELECT orgid, lichessid, exp FROM memberships WHERE orgid = $1",
            &[&org_id],
        )?;
        Ok(extract_one_membership(&rows))
    }

    fn get_member_for_lichess_id(&self, lichess_id: &str) -> Result<Option<Membership>, ErrorBox> {
        let rows = self.w()?.query(
            "SELECT orgid, lichessid, exp FROM memberships WHERE lichessid = $1",
            &[&lichess_id],
        )?;
        Ok(extract_one_membership(&rows))
    }

    fn lichess_member_has_org(&self, lichess_id: &str) -> Result<bool, ErrorBox> {
        self.get_member_for_lichess_id(lichess_id)
            .map(|member| member.is_some())
    }

    fn remove_membership(&self, org_id: &str) -> Result<u64, ErrorBox> {
        let result = self
            .w()?
            .execute("DELETE FROM memberships WHERE orgid = $1", &[&org_id])?;
        Ok(result)
    }

    fn get_members(&self) -> Result<Vec<Membership>, ErrorBox> {
        let mut members: Vec<Membership> = vec![];
        for row in self
            .w()?
            .query("SELECT orgid, lichessid, exp FROM memberships", &[])?
        {
            members.push(Membership {
                org_id: row.get(0),
                lichess_id: row.get(1),
                exp_year: row.get(2),
            });
        }
        Ok(members)
    }

    fn get_members_with_at_most_expiry_year(&self, year: i32) -> Result<Vec<Membership>, ErrorBox> {
        let mut members: Vec<Membership> = vec![];
        for row in self.w()?.query(
            "SELECT orgid, lichessid, exp FROM memberships WHERE exp <= $1",
            &[&year],
        )? {
            members.push(Membership {
                org_id: row.get(0),
                lichess_id: row.get(1),
                exp_year: row.get(2),
            });
        }
        Ok(members)
    }

    fn referral_click(&self, lichess_id: &str) -> Result<u64, ErrorBox> {
        let result = self.w()?.execute(
            "INSERT INTO ref (lichessid) VALUES ($1) ON CONFLICT DO NOTHING",
            &[&lichess_id],
        )?;
        Ok(result)
    }

    fn referral_count(&self) -> Result<i64, ErrorBox> {
        let rows = self.w()?.query("SELECT COUNT(*) FROM ref", &[])?;
        Ok(rows.get(0).ok_or("no row returned")?.get(0))
    }
}
