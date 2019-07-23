use postgres::{Client, NoTls};
use serde::Serialize;
use std::sync::RwLock;

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
    fn register_member(
        &self,
        ecf_id: i32,
        lichess_id: &str,
        exp_year: i32,
    ) -> Result<u64, postgres::Error>;
    fn get_member_for_ecf_id(&self, ecf_id: i32) -> Result<Option<Membership>, postgres::Error>;
    fn get_member_for_lichess_id(
        &self,
        lichess_id: &str,
    ) -> Result<Option<Membership>, postgres::Error>;
    fn lichess_member_has_ecf(&self, lichess_id: &str) -> Result<bool, postgres::Error>;
    fn remove_membership(&self, ecf_id: i32) -> Result<u64, postgres::Error>;
    fn get_members(&self) -> Result<Vec<Membership>, postgres::Error>;
    fn get_members_with_at_most_expiry_year(
        &self,
        year: i32,
    ) -> Result<Vec<Membership>, postgres::Error>;
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

// TODO: [technical debt] replace .unwrap() by ? and still make it all compile
impl EcfDbClient for RwLock<Client> {
    fn register_member(
        &self,
        ecf_id: i32,
        lichess_id: &str,
        exp_year: i32,
    ) -> Result<u64, postgres::Error> {
        let mut client = self.write().unwrap();
        client.execute(
            "DELETE FROM memberships WHERE ecfid = $1 OR lichessid = $2",
            &[&ecf_id, &lichess_id],
        )?;
        client.execute(
            "INSERT INTO memberships (ecfid, lichessid, exp) VALUES ($1, $2, $3);",
            &[&ecf_id, &lichess_id, &exp_year],
        )
    }

    fn get_member_for_ecf_id(&self, ecf_id: i32) -> Result<Option<Membership>, postgres::Error> {
        let rows = self.write().unwrap().query(
            "SELECT ecfid, lichessid, exp FROM memberships WHERE ecfid = $1",
            &[&ecf_id],
        )?;
        Ok(extract_one_membership(&rows))
    }

    fn get_member_for_lichess_id(
        &self,
        lichess_id: &str,
    ) -> Result<Option<Membership>, postgres::Error> {
        let rows = self.write().unwrap().query(
            "SELECT ecfid, lichessid, exp FROM memberships WHERE lichessid = $1",
            &[&lichess_id],
        )?;
        Ok(extract_one_membership(&rows))
    }

    fn lichess_member_has_ecf(&self, lichess_id: &str) -> Result<bool, postgres::Error> {
        self.get_member_for_lichess_id(lichess_id)
            .map(|member| member.is_some())
    }

    fn remove_membership(&self, ecf_id: i32) -> Result<u64, postgres::Error> {
        self.write()
            .unwrap()
            .execute("DELETE FROM memberships WHERE ecfid = $1", &[&ecf_id])
    }

    fn get_members(&self) -> Result<Vec<Membership>, postgres::Error> {
        let mut members: Vec<Membership> = vec![];
        for row in self
            .write()
            .unwrap()
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

    fn get_members_with_at_most_expiry_year(
        &self,
        year: i32,
    ) -> Result<Vec<Membership>, postgres::Error> {
        let mut members: Vec<Membership> = vec![];
        for row in self.write().unwrap().query(
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
