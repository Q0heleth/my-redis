use my_redis::client::Client;

#[tokio::main]
pub async fn main() -> my_redis::Result<()>{
    let mut client = Client::new("127.0.0.1:36379").await?;
    for _ in 0..500 {
    client.set("hello","world".into()).await?;
    if let Some(resp) = client.get("hello").await?{
        println!("get {:?}",resp);
    }else {
        println!("key not exist");
    }
}
    Ok(())
}