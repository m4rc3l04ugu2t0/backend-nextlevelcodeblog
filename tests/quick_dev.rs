use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn quick_dev() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:3000/api")?;

    hc.do_post(
        "/auth/register",
        json!({
          "name": "John Doe",
          "email": "testee@gmal.com",
          "password": "123456",
          "passwordConfirm": "123456",
        }),
    )
    .await?
    .print()
    .await?;

    hc.do_post(
        "/auth/login",
        json!({
          "email": "testee@gmal.com",
          "password": "123456",
        }),
    )
    .await?
    .print()
    .await?;

    // hc.do_get("/auth/verify?token=0194b49c-87a4-72e2-9e7b-9b9a3bde3d48")
    //     .await?
    //     .print()
    //     .await?;

    // hc.do_post("/auth/forgot-password", json!({"email": "testee@gmal.com"}))
    //     .await?
    //     .print()
    //     .await?;

    // hc.do_post(
    //     "/auth/reset-password",
    //     json!({
    //         "token": "0194b756-1154-7c22-8667-7d96fc09c5d6",
    //         "new_password": "newpass",
    //         "new_password_confirm": "newpass",
    //     }),
    // )
    // .await?
    // .print()
    // .await?;

    // let res = hc.do_get("/auth/me").await?;
    // res.print().await?;

    Ok(())
}
