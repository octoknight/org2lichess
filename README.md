## A bridge between chess organizations and Lichess

People can log in to this platform with their Lichess account, and are then asked to fill in
the information that identifies them to their chess club/organization/federation. After the
verification, the player will be automatically added to the organization's Lichess team, so they
can play in that team's tournaments on Lichess and post in the team's forums.
They will also be automatically kicked again if their membership expries and they didn't renew.

### Setup

org2lichess uses PostgreSQL. Set up the necessary tables in your database:

```sql
create table memberships (orgid varchar not null primary key, lichessid varchar not null unique, exp integer not null);

create table ref (lichessid varchar not null primary key);
```

Copy `Config.default.toml` to `Config.toml` and fill in the values. Guidance for this is given
in `Config.default.toml`. For the `[azolve]` configuration block: the assumption here is that
your organization uses Azolve GoMembership to manage memberships (which is the case for the
English Chess Federation, for which this was originally written). If that's not the case for
your organization, you'll have to modify `src/azolve.rs` to use APIs appropriate for your
membership management system.

Copy `Rocket.default.toml` to `Rocket.toml` and fill in `secret_key`. You can generate a key with `openssl rand -base64 32`.

To run it, simply run with cargo: `cargo run --release`
