use postgres::{Client, NoTls};

struct Membership {
    ecf_id: i32,
    lichess_id: String,
    exp_year: i32,
}

fn connect(connection_string: &str) -> Result<Client, postgres::Error> {
    Client::connect(connection_string, NoTls)
}

trait EcfDbClient {
    fn register_member(
        &mut self,
        ecf_id: i32,
        lichess_id: String,
        exp_year: i32,
    ) -> Result<u64, postgres::Error>;
    fn get_member_for_ecf_id(&mut self, ecf_id: i32)
        -> Result<Option<Membership>, postgres::Error>;
    fn get_member_for_lichess_id(
        &mut self,
        lichess_id: String,
    ) -> Result<Option<Membership>, postgres::Error>;
    fn remove_membership(&mut self, ecf_id: i32) -> Result<u64, postgres::Error>;
    fn update_expiry(&mut self, ecf_id: i32, new_exp_year: i32) -> Result<u64, postgres::Error>;
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

impl EcfDbClient for Client {
    fn register_member(
        &mut self,
        ecf_id: i32,
        lichess_id: String,
        exp_year: i32,
    ) -> Result<u64, postgres::Error> {
        self.execute(
            "INSERT INTO memberships (ecfid, lichessid, exp) VALUES ($1, $2, $3)",
            &[&ecf_id, &lichess_id, &exp_year],
        )
    }

    fn get_member_for_ecf_id(
        &mut self,
        ecf_id: i32,
    ) -> Result<Option<Membership>, postgres::Error> {
        let rows = self.query(
            "SELECT ecfid, lichessid, exp FROM memberships WHERE ecfid = $1",
            &[&ecf_id],
        )?;
        Ok(extract_one_membership(&rows))
    }

    fn get_member_for_lichess_id(
        &mut self,
        lichess_id: String,
    ) -> Result<Option<Membership>, postgres::Error> {
        let rows = self.query(
            "SELECT ecfid, lichessid, exp FROM memberships WHERE lichessid = $1",
            &[&lichess_id],
        )?;
        Ok(extract_one_membership(&rows))
    }

    fn remove_membership(&mut self, ecf_id: i32) -> Result<u64, postgres::Error> {
        self.execute("DELETE FROM memberships WHERE ecfid = $1", &[&ecf_id])
    }

    fn update_expiry(&mut self, ecf_id: i32, new_exp_year: i32) -> Result<u64, postgres::Error> {
        self.execute(
            "UPDATE memberships SET exp = $1 WHERE ecfid = $2",
            &[&ecf_id, &new_exp_year],
        )
    }
}
