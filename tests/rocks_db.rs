use vade::AsyncResult;
use vade_didcomm::{read_db, write_db};

#[tokio::test]
async fn can_use_rocks_db() -> AsyncResult<()> {
    write_db("test1", "helloooo");
    let result = read_db("test1")?;

    assert_eq!(result, "helloooo");

    return Ok(());
}
