use std::collections::HashMap;
use anyhow::anyhow;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref TOKEN_REGEX: Regex = Regex::new(r#"name="logintoken"\s+value="([^"]+)""#).unwrap();
}

pub async fn login_as_guest(client: &reqwest::Client) -> anyhow::Result<()> {
    let login_url = "https://moodle.hwr-berlin.de/login/index.php";
    let res = client.get(login_url).send().await?;
    let body = res.text().await?;
    
    let token = TOKEN_REGEX.captures(&body)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .ok_or_else(|| anyhow!("could not find logintoken in Moodle login page"))?;
        
    let mut params = HashMap::new();
    params.insert("logintoken", token);
    params.insert("username", "guest");
    params.insert("password", "guest");
    
    let res = client.post(login_url)
        .form(&params)
        .send()
        .await?;
        
    if !res.status().is_success() {
        return Err(anyhow!("Moodle guest login POST failed: status {}", res.status()));
    }
    
    Ok(())
}

pub async fn get_moodle(client: &reqwest::Client, url: &str) -> anyhow::Result<reqwest::Response> {
    let res = client.get(url).send().await?;
    
    if res.url().path().contains("/login/index.php") {
        tracing::info!("Redirected to login page. Performing guest login...");
        login_as_guest(client).await?;
        
        tracing::info!("Retrying request to {} after guest login", url);
        let res = client.get(url).send().await?;
        if res.url().path().contains("/login/index.php") {
            return Err(anyhow!("Redirected to login page even after guest login"));
        }
        Ok(res)
    } else {
        Ok(res)
    }
}
