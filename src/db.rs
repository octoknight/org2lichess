use crate::types::*;
use postgres::NoTls;
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;
use serde::Serialize;

#[derive(Serialize)]
pub struct Membership {
    pub org_id: String,
    pub lichess_id: String,
    pub exp_year: i32,
}

type DbPool = Pool<PostgresConnectionManager<NoTls>>;
type DbConnection = PooledConnection<PostgresConnectionManager<NoTls>>;

#[derive(Clone)]
pub struct OrgDbClient(DbPool);

pub fn connect(connection_options: &str) -> Result<OrgDbClient, ErrorBox> {
    let manager = PostgresConnectionManager::new(connection_options.parse()?, NoTls);
    let pool = Pool::new(manager)?;
    Ok(OrgDbClient(pool))
}

fn extract_one_membership(rows: &[postgres::row::Row]) -> Option<Membership> {
    rows.get(0).map(|row| Membership {
        org_id: row.get(0),
        lichess_id: row.get(1),
        exp_year: row.get(2),
    })
}

impl OrgDbClient {
    fn w(&self) -> Result<DbConnection, ErrorBox> {
        Ok(self.0.get()?)
    }

    pub fn register_member(
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
            "INSERT INTO memberships (orgid, lichessid, exp) VALUES ($1, $2, $3)",
            &[&org_id, &lichess_id, &exp_year],
        )?;
        Ok(result)
    }

    pub fn get_member_for_org_id(&self, org_id: &str) -> Result<Option<Membership>, ErrorBox> {
        let rows = self.w()?.query(
            "SELECT orgid, lichessid, exp FROM memberships WHERE orgid = $1",
            &[&org_id],
        )?;
        Ok(extract_one_membership(&rows))
    }

    pub fn get_member_for_lichess_id(
        &self,
        lichess_id: &str,
    ) -> Result<Option<Membership>, ErrorBox> {
        let rows = self.w()?.query(
            "SELECT orgid, lichessid, exp FROM memberships WHERE lichessid = $1",
            &[&lichess_id],
        )?;
        Ok(extract_one_membership(&rows))
    }

    pub fn lichess_member_has_org(&self, lichess_id: &str) -> Result<bool, ErrorBox> {
        self.get_member_for_lichess_id(lichess_id)
            .map(|member| member.is_some())
    }

    pub fn remove_membership(&self, org_id: &str) -> Result<u64, ErrorBox> {
        let result = self
            .w()?
            .execute("DELETE FROM memberships WHERE orgid = $1", &[&org_id])?;
        Ok(result)
    }

    pub fn remove_membership_by_lichess_id(&self, lichess_id: &str) -> Result<u64, ErrorBox> {
        let result = self.w()?.execute(
            "DELETE FROM memberships WHERE lichessid = $1",
            &[&lichess_id],
        )?;
        Ok(result)
    }

    pub fn get_members(&self) -> Result<Vec<Membership>, ErrorBox> {
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

    pub fn get_members_with_at_most_expiry_year(
        &self,
        year: i32,
    ) -> Result<Vec<Membership>, ErrorBox> {
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

    pub fn referral_click(&self, lichess_id: &str) -> Result<u64, ErrorBox> {
        let result = self.w()?.execute(
            "INSERT INTO ref (lichessid) VALUES ($1) ON CONFLICT DO NOTHING",
            &[&lichess_id],
        )?;
        Ok(result)
    }

    pub fn referral_count(&self) -> Result<i64, ErrorBox> {
        let rows = self.w()?.query("SELECT COUNT(*) FROM ref", &[])?;
        Ok(rows.get(0).ok_or("no row returned")?.get(0))
    }
}
