use vade_didcomm::{read_db, write_db, SyncResult};

#[test]
fn can_use_rocks_db() -> SyncResult<()> {
    let _ = write_db("test1", "helloooo");
    let result = read_db("test1")?;

    assert_eq!(result, "helloooo");

    return Ok(());
}
